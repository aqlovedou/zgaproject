"""组卷、评分、错题本与间隔复习的行为测试。"""
import random
import sys
import tempfile
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from az900.bank import Bank
from az900.grading import domain_breakdown, passed, scaled_score, topic_weakness
from az900.models import Answered, DOMAINS
from az900.session import parse_choice
from az900.storage import Store

BANK = Bank.load()


def test_exam_follows_domain_weights():
    exam = BANK.build_exam(50, random.Random(7))
    assert len(exam) == 50
    counts = {d: 0 for d in DOMAINS}
    for q in exam:
        counts[q.domain] += 1
    for key, meta in DOMAINS.items():
        expected = 50 * meta["weight"]
        assert abs(counts[key] - expected) <= 3, f"{key} 抽题 {counts[key]}，期望约 {expected}"


def test_exam_has_no_duplicates():
    exam = BANK.build_exam(50, random.Random(3))
    assert len({q.id for q in exam}) == 50


def test_exam_respects_exclusions():
    excluded = {q.id for q in BANK.questions[:20]}
    exam = BANK.build_exam(30, random.Random(1), exclude=excluded)
    assert not ({q.id for q in exam} & excluded)


def test_scoring_and_pass_line():
    assert scaled_score(35, 50) == 700
    assert passed(scaled_score(35, 50))
    assert not passed(scaled_score(34, 50))
    assert scaled_score(0, 0) == 0


def test_multi_answer_requires_exact_match():
    multi = next(q for q in BANK.questions if q.type == "multi")
    assert multi.is_correct(list(multi.answer))
    assert multi.is_correct(list(reversed(multi.answer)))
    assert not multi.is_correct([multi.answer[0]])


def test_parse_choice_variants():
    q = next(x for x in BANK.questions if x.type == "multi" and len(x.answer) == 2)
    a, b = q.answer
    assert parse_choice(f"{a}{b}", q) == [a, b]
    assert parse_choice(f"{a.lower()}, {b.lower()}", q) == [a, b]
    assert parse_choice(f"{a}{a}", q) is None      # 重复选项非法
    assert parse_choice("Z", q) is None            # 不存在的选项
    assert parse_choice("", q) is None
    single = next(x for x in BANK.questions if x.type == "single")
    assert parse_choice("AB", single) is None      # 单选题不能多选


def test_domain_breakdown_and_weakness():
    questions = BANK.build_exam(30, random.Random(5))
    answers = {q.id: Answered(q.id, list(q.answer), i % 2 == 0, 5.0)
               for i, q in enumerate(questions)}
    stats = domain_breakdown(questions, answers)
    assert sum(s["total"] for s in stats.values()) == 30
    assert all(0.0 <= s["rate"] <= 1.0 for s in stats.values())
    weak = topic_weakness(questions, answers)
    assert all(wrong > 0 for _, wrong, _ in weak)


def test_wrongbook_records_and_masters():
    with tempfile.TemporaryDirectory() as tmp:
        store = Store(Path(tmp))
        q = BANK.questions[0]
        store.add_wrong(q.id, ["B"], q.answer, q.domain, q.topic)
        assert len(store.wrong_entries()) == 1
        entry = store.wrong_entries()[0]
        assert entry["wrong_count"] == 1 and entry["box"] == 1

        # 连续答对：盒子升级，达到条件后视为攻克
        for _ in range(4):
            store.mark_review_result(q.id, True, list(q.answer))
        assert store.wrong_entries()[0:] == [] or store.wrong_entries() == []
        assert store.wrongbook["entries"][q.id]["mastered"] is True

        # 再答错则重新回到未攻克
        store.mark_review_result(q.id, False, ["B"])
        assert store.wrongbook["entries"][q.id]["mastered"] is False
        assert store.wrongbook["entries"][q.id]["box"] == 1
        assert len(store.wrong_entries()) == 1


def test_store_persists_across_instances():
    with tempfile.TemporaryDirectory() as tmp:
        q = BANK.questions[1]
        store = Store(Path(tmp))
        store.add_wrong(q.id, ["A"], q.answer, q.domain, q.topic)
        store.record_session("exam", {"score": 720, "total": 50, "correct": 36,
                                      "elapsed_sec": 100, "answers": {}, "questions": []})
        store.save()

        reloaded = Store(Path(tmp))
        assert len(reloaded.wrong_entries()) == 1
        assert reloaded.progress["exams"][0]["score"] == 720


def test_due_entries_respects_schedule():
    with tempfile.TemporaryDirectory() as tmp:
        store = Store(Path(tmp))
        q = BANK.questions[2]
        store.add_wrong(q.id, ["A"], q.answer, q.domain, q.topic)
        assert len(store.due_entries()) == 1       # 刚答错，立即到期
        store.mark_review_result(q.id, True, list(q.answer))
        assert len(store.due_entries()) == 0       # 答对后推迟到 3 小时后
