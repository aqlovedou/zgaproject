"""题库完整性测试：保证每道题都可答、可判分、有解析和官方出处。"""
import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).resolve().parent.parent))

from az900.bank import Bank
from az900.models import DOMAINS

BANK = Bank.load()


def test_bank_not_empty():
    assert len(BANK) >= 150, f"题量不足：{len(BANK)}"


def test_ids_unique_and_prefixed():
    ids = [q.id for q in BANK.questions]
    assert len(ids) == len(set(ids)), "存在重复题号"
    for q in BANK.questions:
        assert q.id.startswith(("D1-", "D2-", "D3-")), q.id
        assert int(q.id.split("-")[0][1]) == q.domain_no, f"{q.id} 题号与领域不符"


def test_every_domain_covered():
    for key in DOMAINS:
        assert len(BANK.by_domain(key)) >= 30, f"{key} 题量不足"


def test_answers_valid():
    for q in BANK.questions:
        assert q.answer, q.id
        assert all(k in q.options for k in q.answer), q.id
        assert len(q.options) >= 2, q.id
        if q.type == "multi":
            assert len(q.answer) >= 2, q.id
        else:
            assert len(q.answer) == 1, q.id


def test_every_wrong_option_has_reason():
    for q in BANK.questions:
        for key in q.options:
            if key not in q.answer:
                assert q.option_reasons.get(key), f"{q.id} 选项 {key} 缺少错误原因"


def test_every_question_has_official_reference():
    for q in BANK.questions:
        assert q.reference_url.startswith("https://learn.microsoft.com"), q.id
        assert q.reference_title, q.id


def test_explanations_are_substantive():
    for q in BANK.questions:
        assert len(q.explanation) >= 40, f"{q.id} 解析过短"


def test_truefalse_answers_not_all_same():
    tf = [q for q in BANK.questions if q.type == "truefalse"]
    answers = {q.answer[0] for q in tf}
    assert len(answers) > 1, "判断题答案全部相同，会被猜中"
