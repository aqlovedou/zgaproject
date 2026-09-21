"""交互式答题流程：模拟考 / 专项练习 / 错题复习。"""
from __future__ import annotations

import time
from typing import Any, Callable, Iterable

from . import ui
from .grading import domain_breakdown, passed, scaled_score, topic_weakness
from .models import Answered, PASS_SCORE, Question
from .storage import Store

Prompt = Callable[[str], str]


class QuitSession(Exception):
    """用户中途退出。"""


def _ask(prompt: Prompt, text: str) -> str:
    try:
        return prompt(text).strip()
    except (EOFError, KeyboardInterrupt):
        raise QuitSession from None


def render_question(q: Question, index: int, total: int, elapsed_hint: str = "") -> str:
    head = f"第 {index}/{total} 题  [{q.type_label}]  领域{q.domain_no}·{q.topic}"
    if elapsed_hint:
        head += f"   {ui.dim(elapsed_hint)}"
    lines = [ui.rule(), ui.bold(head), "", q.question, ""]
    for key in sorted(q.options):
        lines.append(f"  {ui.blue(key)}. {q.options[key]}")
    if q.type == "multi":
        lines.append("")
        lines.append(ui.dim(f"（多选题，请选 {len(q.answer)} 项，例如输入 AC）"))
    return "\n".join(lines)


def render_feedback(q: Question, picked: list[str], correct: bool) -> str:
    lines = []
    if correct:
        lines.append(ui.green("✅ 回答正确"))
    else:
        lines.append(ui.red("❌ 回答错误"))
        lines.append(f"   你的答案：{ui.red('、'.join(picked) or '（未作答）')}")
    lines.append(f"   正确答案：{ui.green('、'.join(q.answer))}  —— {q.answer_text}")
    lines.append("")
    lines.append(f"{ui.bold('【解析】')} {q.explanation}")
    if not correct:
        wrong_keys = [k for k in picked if k not in q.answer]
        missed = [k for k in q.answer if k not in picked]
        if wrong_keys:
            lines.append(f"{ui.bold('【你为什么错】')}")
            for k in wrong_keys:
                lines.append(f"   {ui.red('✗')} {k}. {q.options[k]}")
                lines.append(f"      {q.wrong_reason(k)}")
        if missed and q.type == "multi":
            lines.append(f"{ui.bold('【你漏掉的正确项】')}")
            for k in missed:
                lines.append(f"   {ui.green('✓')} {k}. {q.options[k]}")
    lines.append(f"{ui.bold('【官方出处】')} {q.reference_title}")
    lines.append(f"   {ui.blue(q.reference_url)}")
    return "\n".join(lines)


def parse_choice(raw: str, q: Question) -> list[str] | None:
    """把 'a c' / 'AC' / 'A,C' 解析成 ['A','C']，非法返回 None。"""
    keys = [ch.upper() for ch in raw if ch.isalpha()]
    if not keys:
        return None
    if len(set(keys)) != len(keys):
        return None
    if any(k not in q.options for k in keys):
        return None
    if q.type in {"single", "truefalse"} and len(keys) != 1:
        return None
    return keys


def run_quiz(questions: list[Question], store: Store, kind: str,
             prompt: Prompt = input, out: Callable[[str], None] = print,
             time_limit_min: int | None = None,
             show_feedback: bool = True,
             on_result: Callable[[Question, list[str], bool], None] | None = None,
             ) -> dict[str, Any]:
    """通用答题循环。show_feedback=False 用于模拟考（考完统一讲评）。"""
    answers: dict[str, Answered] = {}
    started = time.time()
    total = len(questions)

    for i, q in enumerate(questions, 1):
        if time_limit_min:
            left = time_limit_min * 60 - (time.time() - started)
            if left <= 0:
                out(ui.red("\n⏰ 时间到！未作答的题目按错误计分。"))
                break
            hint = f"剩余 {int(left // 60)}:{int(left % 60):02d}"
        else:
            hint = ""

        out(render_question(q, i, total, hint))
        q_started = time.time()
        while True:
            raw = _ask(prompt, "你的答案（q=退出，s=跳过）> ")
            if raw.lower() in {"q", "quit", "exit"}:
                raise QuitSession
            if raw.lower() in {"s", "skip"}:
                picked: list[str] = []
                break
            parsed = parse_choice(raw, q)
            if parsed is None:
                out(ui.yellow("输入无效，请重新输入选项字母。"))
                continue
            picked = parsed
            break

        correct = q.is_correct(picked)
        answers[q.id] = Answered(q.id, picked, correct, time.time() - q_started)

        if kind == "review":
            store.mark_review_result(q.id, correct, picked)
        elif not correct:
            store.add_wrong(q.id, picked, q.answer, q.domain, q.topic)

        if on_result:
            on_result(q, picked, correct)
        if show_feedback:
            out("")
            out(render_feedback(q, picked, correct))
            out("")

    elapsed = time.time() - started
    correct_count = sum(1 for a in answers.values() if a.correct)
    result = {
        "kind": kind,
        "at": started,
        "total": total,
        "answered": len(answers),
        "correct": correct_count,
        "score": scaled_score(correct_count, total),
        "elapsed_sec": round(elapsed, 1),
        "questions": [q.id for q in questions],
        "answers": {qid: a.to_dict() for qid, a in answers.items()},
    }
    store.record_session(kind, result)
    store.save()
    return result


def render_report(questions: list[Question], result: dict[str, Any],
                  answers: dict[str, Answered]) -> str:
    score = result["score"]
    ok = passed(score)
    lines = [ui.title("成绩单")]
    verdict = ui.green(f"{score} 分 —— 达标 ✅") if ok else ui.red(f"{score} 分 —— 未达标 ❌（及格线 {PASS_SCORE}）")
    lines.append(f"折算分数：{ui.bold(verdict)}")
    lines.append(f"答对 {result['correct']}/{result['total']} 题"
                 f"，用时 {int(result['elapsed_sec'] // 60)} 分 {int(result['elapsed_sec'] % 60)} 秒")
    lines.append("")
    lines.append(ui.bold("各领域得分（官方按领域分别评估）"))
    for key, row in sorted(domain_breakdown(questions, answers).items(), key=lambda kv: kv[1]["no"]):
        if not row["total"]:
            continue
        lines.append(f"  领域{row['no']} {row['name_cn']:<14} "
                     f"{ui.bar(row['rate'])} {row['correct']:>2}/{row['total']:<2} "
                     f"({row['rate']:.0%})")
    weak = topic_weakness(questions, answers)
    if weak:
        lines.append("")
        lines.append(ui.bold("最该补的考点"))
        for topic, wrong, tot in weak[:8]:
            lines.append(f"  {ui.red('•')} {topic}：错 {wrong}/{tot}")
    return "\n".join(lines)
