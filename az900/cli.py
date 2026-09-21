"""命令行入口：模拟考、专项练习、错题本、进度统计、学习计划。"""
from __future__ import annotations

import argparse
import random
import sys
from typing import Any

from . import plan as plan_mod
from . import ui
from .bank import Bank, BankError
from .grading import domain_breakdown, passed, scaled_score
from .models import DEFAULT_EXAM_COUNT, DEFAULT_EXAM_MINUTES, DOMAIN_BY_NO, DOMAINS, Answered
from .session import QuitSession, render_feedback, render_report, run_quiz
from .storage import Store


def _answers_from_result(result: dict[str, Any]) -> dict[str, Answered]:
    return {
        qid: Answered(qid, a["picked"], a["correct"], a["seconds"], a.get("at", 0))
        for qid, a in result["answers"].items()
    }


def cmd_exam(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    count = min(args.count, len(bank))
    questions = bank.build_exam(count, random.Random(args.seed))
    minutes = args.time if args.time is not None else max(5, round(count * 0.9))
    print(ui.title(f"全真模拟考：{count} 题 / {minutes} 分钟 / 及格线 700 分"))
    print(ui.dim("考试中不显示答案，交卷后统一讲评。输入 q 可中途退出。\n"))
    try:
        result = run_quiz(questions, store, "exam", time_limit_min=minutes,
                          show_feedback=False)
    except QuitSession:
        print(ui.yellow("\n已退出，本次成绩不计入。"))
        return 1

    answers = _answers_from_result(result)
    print(render_report(questions, result, answers))

    wrong = [q for q in questions if not (q.id in answers and answers[q.id].correct)]
    if wrong:
        print(ui.title(f"逐题讲评：{len(wrong)} 道错题（已全部收入错题本）"))
        for q in wrong:
            got = answers.get(q.id)
            picked = got.picked if got else []
            print(ui.rule())
            print(ui.bold(f"[{q.id}] {q.topic}"))
            print(q.question)
            for key in sorted(q.options):
                print(f"  {key}. {q.options[key]}")
            print("")
            print(render_feedback(q, picked, False))
            print("")
        print(ui.yellow(f"→ 下一步：python3 az900.py wrong --review  （复习这 {len(wrong)} 道错题）"))
    else:
        print(ui.green("\n🎉 全对！"))
    return 0


def cmd_practice(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    domain = DOMAIN_BY_NO.get(args.domain) if args.domain else None
    pool = bank.filter(domain=domain, topic=args.topic, keyword=args.keyword)
    if not pool:
        print(ui.red("没有匹配的题目。用 `python3 az900.py topics` 看所有考点。"))
        return 1
    if args.only_new:
        done = {qid for s in store.progress.get("practices", []) + store.progress.get("exams", [])
                for qid in s.get("answers", {})}
        fresh = [q for q in pool if q.id not in done]
        pool = fresh or pool
    rng = random.Random(args.seed)
    rng.shuffle(pool)
    questions = pool[: args.count] if args.count else pool

    label = f"领域{args.domain}" if args.domain else (args.topic or args.keyword or "全部考点")
    print(ui.title(f"专项练习：{label}（{len(questions)} 题，答完即时讲评）"))
    try:
        result = run_quiz(questions, store, "practice", show_feedback=True)
    except QuitSession:
        print(ui.yellow("\n已退出，已作答的部分已保存。"))
        return 0
    print(render_report(questions, result, _answers_from_result(result)))
    remaining = len(store.wrong_entries())
    if remaining:
        print(ui.yellow(f"\n错题本当前待攻克：{remaining} 题 → python3 az900.py wrong --review"))
    return 0


def cmd_wrong(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    entries = store.wrong_entries(include_mastered=args.all)
    if not entries:
        print(ui.green("错题本是空的 👍 先去做一套：python3 az900.py exam"))
        return 0

    if not args.review:
        print(ui.title(f"错题本：{len(entries)} 题待攻克"))
        for e in entries:
            q = bank.get(e["qid"])
            if q is None:
                continue
            flag = ui.green("已攻克") if e.get("mastered") else ui.red(f"错 {e['wrong_count']} 次")
            box = e.get("box", 1)
            print(f"{ui.rule()}")
            print(f"{ui.bold('[' + q.id + ']')} {q.topic}  {flag}  {ui.dim('掌握度 ' + '●' * box + '○' * (5 - box))}")
            print(f"  {q.question.splitlines()[0][:70]}")
            print(f"  正确答案：{ui.green('、'.join(q.answer))}   你曾选：{ui.red('、'.join(e.get('last_picked') or []) or '—')}")
            if args.verbose:
                print(f"  解析：{q.explanation}")
                print(f"  出处：{ui.blue(q.reference_url)}")
        print("")
        print(ui.yellow("开始复习： python3 az900.py wrong --review"))
        return 0

    due = store.due_entries()
    pending = due or entries
    if due:
        print(ui.dim(f"（按间隔复习算法，本轮到期 {len(due)} 题）"))
    questions = [q for q in (bank.get(e["qid"]) for e in pending) if q is not None]
    if args.count:
        questions = questions[: args.count]
    random.Random(args.seed).shuffle(questions)

    print(ui.title(f"错题复习：{len(questions)} 题（连续答对 2 次且掌握度达 4 级即攻克）"))
    try:
        result = run_quiz(questions, store, "review", show_feedback=True)
    except QuitSession:
        print(ui.yellow("\n已退出，进度已保存。"))
        return 0
    left = len(store.wrong_entries())
    print(render_report(questions, result, _answers_from_result(result)))
    if left:
        print(ui.yellow(f"\n还剩 {left} 题没攻克，再来一轮： python3 az900.py wrong --review"))
    else:
        print(ui.green("\n🎉 错题本清空了！去做一套全真模拟验收： python3 az900.py exam"))
    return 0


def cmd_stats(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    exams = store.progress.get("exams", [])
    practices = store.progress.get("practices", [])
    print(ui.title("学习进度"))

    all_answers: dict[str, list[bool]] = {}
    for s in exams + practices:
        for qid, a in s.get("answers", {}).items():
            all_answers.setdefault(qid, []).append(a["correct"])
    seen = len(all_answers)
    print(f"题库总量：{ui.bold(str(len(bank)))} 题   已练过：{ui.bold(str(seen))} 题"
          f"（覆盖率 {seen / len(bank):.0%}）")
    total_attempts = sum(len(v) for v in all_answers.values())
    total_right = sum(sum(v) for v in all_answers.values())
    if total_attempts:
        print(f"累计作答：{total_attempts} 次，正确率 {ui.bold(f'{total_right / total_attempts:.0%}')}")

    # 各领域累计正确率
    per_domain = {k: [0, 0] for k in DOMAINS}
    for qid, results in all_answers.items():
        q = bank.get(qid)
        if q is None:
            continue
        per_domain[q.domain][0] += sum(results)
        per_domain[q.domain][1] += len(results)
    print("")
    print(ui.bold("各领域累计正确率（目标：每个领域都 ≥ 80%）"))
    for key, meta in sorted(DOMAINS.items(), key=lambda kv: kv[1]["no"]):
        right, tot = per_domain[key]
        rate = right / tot if tot else 0.0
        mark = ui.green("达标") if rate >= 0.8 and tot >= 10 else ui.yellow("待加强")
        print(f"  领域{meta['no']} {meta['name_cn']:<14} {ui.bar(rate)} {right:>3}/{tot:<3} ({rate:.0%}) {mark if tot else ui.dim('未练')}")

    if exams:
        print("")
        print(ui.bold("模拟考记录（及格线 700）"))
        for i, s in enumerate(exams[-10:], 1):
            score = s["score"]
            tag = ui.green("达标 ✅") if passed(score) else ui.red("未达标")
            print(f"  第 {len(exams) - min(len(exams), 10) + i} 次  {s['correct']:>2}/{s['total']:<2}  "
                  f"{ui.bold(str(score)):>4} 分  {tag}  用时 {int(s['elapsed_sec'] // 60)} 分")
        best = max(s["score"] for s in exams)
        last2 = [s["score"] for s in exams[-2:]]
        print(f"  最好成绩：{ui.bold(str(best))} 分")
        if len(last2) == 2 and all(s >= 800 for s in last2):
            print(ui.green("  ✅ 连续两次 ≥800 分，达到可以上考场的状态。"))
        elif passed(best):
            print(ui.yellow("  ⚠️ 已及格过，但还不稳。目标是连续两次 ≥800。"))
        else:
            print(ui.yellow("  ⚠️ 还没及格过，先把错题本清空再考。"))

    entries = store.wrong_entries(include_mastered=True)
    if entries:
        mastered = sum(1 for e in entries if e.get("mastered"))
        print("")
        print(ui.bold("错题本"))
        print(f"  收录 {len(entries)} 题，已攻克 {ui.green(str(mastered))} 题，"
              f"待攻克 {ui.red(str(len(entries) - mastered))} 题")
        weak: dict[str, int] = {}
        for e in entries:
            if not e.get("mastered"):
                weak[e["topic"]] = weak.get(e["topic"], 0) + e.get("wrong_count", 1)
        if weak:
            print("  最薄弱考点：" + "、".join(
                f"{t}({n})" for t, n in sorted(weak.items(), key=lambda kv: -kv[1])[:5]))
    return 0


def cmd_topics(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    print(ui.title(f"考点清单（题库共 {len(bank)} 题）"))
    topics = bank.topics()
    for key, meta in sorted(DOMAINS.items(), key=lambda kv: kv[1]["no"]):
        pool = bank.by_domain(key)
        head = ui.bold("领域{} {}".format(meta["no"], meta["name_cn"]))
        note = ui.dim("官方权重 {:.0%} · 本题库 {} 题".format(meta["weight"], len(pool)))
        print(f"\n{head} {note}")
        for t in topics[key]:
            n = len([q for q in pool if q.topic == t])
            print("  • {} {}".format(t, ui.dim(f"({n} 题)")))
    print("")
    print(ui.dim("按考点练习： python3 az900.py practice --topic 存储"))
    return 0


def cmd_plan(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    print(plan_mod.render_day(args.day) if args.day else plan_mod.render_all())
    return 0


def cmd_show(args: argparse.Namespace, bank: Bank, store: Store) -> int:
    q = bank.get(args.qid.upper())
    if q is None:
        print(ui.red(f"找不到题号 {args.qid}"))
        return 1
    print(ui.title(f"[{q.id}] 领域{q.domain_no}·{q.topic}"))
    print(q.question)
    for key in sorted(q.options):
        mark = ui.green(" ←正确") if key in q.answer else ""
        print(f"  {key}. {q.options[key]}{mark}")
    print("")
    print(render_feedback(q, list(q.answer), True))
    if q.option_reasons:
        print(f"\n{ui.bold('【各错误选项为什么错】')}")
        for key, reason in sorted(q.option_reasons.items()):
            print(f"   {ui.red('✗')} {key}. {q.options.get(key, '')}\n      {reason}")
    return 0


def build_parser() -> argparse.ArgumentParser:
    p = argparse.ArgumentParser(
        prog="az900",
        description="AZ-900 备考刷题系统：模拟考 / 专项练习 / 错题本 / 进度统计",
    )
    sub = p.add_subparsers(dest="cmd", required=True)

    e = sub.add_parser("exam", help="全真模拟考（按官方领域权重组卷、限时、考后讲评）")
    e.add_argument("--count", type=int, default=DEFAULT_EXAM_COUNT, help="题量，默认 50")
    e.add_argument("--time", type=int, default=None, help=f"限时分钟数，默认按题量折算（50 题≈{DEFAULT_EXAM_MINUTES} 分钟）")
    e.add_argument("--seed", type=int, default=None)
    e.set_defaults(func=cmd_exam)

    pr = sub.add_parser("practice", help="专项练习（即时讲评）")
    pr.add_argument("--domain", type=int, choices=[1, 2, 3], help="按领域：1 云概念 2 架构与服务 3 管理与治理")
    pr.add_argument("--topic", help="按考点关键词，如 存储 / 网络 / 身份")
    pr.add_argument("--keyword", help="按题干关键词搜索，如 ExpressRoute")
    pr.add_argument("--count", type=int, default=None, help="题量，默认全部")
    pr.add_argument("--only-new", action="store_true", help="只练没做过的题")
    pr.add_argument("--seed", type=int, default=None)
    pr.set_defaults(func=cmd_practice)

    w = sub.add_parser("wrong", help="错题本：查看或复习")
    w.add_argument("--review", action="store_true", help="进入复习模式（间隔重复）")
    w.add_argument("--all", action="store_true", help="包含已攻克的题")
    w.add_argument("--verbose", action="store_true", help="列表里直接显示解析和出处")
    w.add_argument("--count", type=int, default=None)
    w.add_argument("--seed", type=int, default=None)
    w.set_defaults(func=cmd_wrong)

    s = sub.add_parser("stats", help="学习进度与达标判断")
    s.set_defaults(func=cmd_stats)

    t = sub.add_parser("topics", help="列出全部考点")
    t.set_defaults(func=cmd_topics)

    pl = sub.add_parser("plan", help="5 天冲刺计划")
    pl.add_argument("--day", type=int, choices=[1, 2, 3, 4, 5])
    pl.set_defaults(func=cmd_plan)

    sh = sub.add_parser("show", help="查看某道题的完整解析，如 show D2-013")
    sh.add_argument("qid")
    sh.set_defaults(func=cmd_show)
    return p


def main(argv: list[str] | None = None) -> int:
    args = build_parser().parse_args(argv)
    try:
        bank = Bank.load()
    except BankError as exc:
        print(ui.red(f"题库有问题：{exc}"), file=sys.stderr)
        return 2
    store = Store()
    try:
        return args.func(args, bank, store)
    except KeyboardInterrupt:
        print(ui.yellow("\n已中断，进度已保存。"))
        store.save()
        return 130
