"""题库加载、校验与组卷。"""
from __future__ import annotations

import json
import random
from pathlib import Path

from .models import DOMAINS, Question

BANK_DIR = Path(__file__).resolve().parent.parent / "data" / "questions"


class BankError(RuntimeError):
    pass


def load_questions(bank_dir: Path | None = None) -> list[Question]:
    directory = bank_dir or BANK_DIR
    files = sorted(directory.glob("*.json"))
    if not files:
        raise BankError(f"题库目录没有任何题目文件: {directory}")

    questions: list[Question] = []
    seen: set[str] = set()
    for path in files:
        raw = json.loads(path.read_text(encoding="utf-8"))
        for item in raw["questions"]:
            q = Question.from_dict(item)
            validate(q, source=path.name)
            if q.id in seen:
                raise BankError(f"题号重复: {q.id} ({path.name})")
            seen.add(q.id)
            questions.append(q)
    return questions


def validate(q: Question, source: str = "") -> None:
    where = f"[{source}] {q.id}"
    if q.domain not in DOMAINS:
        raise BankError(f"{where}: 未知领域 {q.domain}")
    if q.type not in {"single", "multi", "truefalse"}:
        raise BankError(f"{where}: 未知题型 {q.type}")
    if not q.answer:
        raise BankError(f"{where}: 缺少答案")
    for key in q.answer:
        if key not in q.options:
            raise BankError(f"{where}: 答案 {key} 不在选项中")
    if q.type in {"single", "truefalse"} and len(q.answer) != 1:
        raise BankError(f"{where}: {q.type} 只能有一个答案")
    if q.type == "multi" and len(q.answer) < 2:
        raise BankError(f"{where}: 多选题至少要有两个答案")
    if not q.reference_url.startswith("https://learn.microsoft.com"):
        raise BankError(f"{where}: 出处必须是 learn.microsoft.com 官方文档")
    for key in q.options:
        if key not in q.answer and key not in q.option_reasons:
            raise BankError(f"{where}: 错误选项 {key} 缺少「为什么错」的说明")
    if len(q.explanation) < 20:
        raise BankError(f"{where}: 解析过短")


class Bank:
    def __init__(self, questions: list[Question]):
        self.questions = questions
        self.by_id = {q.id: q for q in questions}

    @classmethod
    def load(cls, bank_dir: Path | None = None) -> "Bank":
        return cls(load_questions(bank_dir))

    def __len__(self) -> int:
        return len(self.questions)

    def get(self, qid: str) -> Question | None:
        return self.by_id.get(qid)

    def by_domain(self, domain: str) -> list[Question]:
        return [q for q in self.questions if q.domain == domain]

    def topics(self) -> dict[str, list[str]]:
        out: dict[str, list[str]] = {d: [] for d in DOMAINS}
        for q in self.questions:
            if q.topic not in out[q.domain]:
                out[q.domain].append(q.topic)
        return out

    def filter(self, domain: str | None = None, topic: str | None = None,
               keyword: str | None = None) -> list[Question]:
        pool = self.questions
        if domain:
            pool = [q for q in pool if q.domain == domain]
        if topic:
            pool = [q for q in pool if topic in q.topic]
        if keyword:
            low = keyword.lower()
            pool = [q for q in pool
                    if low in q.question.lower()
                    or low in q.topic.lower()
                    or any(low in k.lower() for k in q.keywords)]
        return pool

    def build_exam(self, count: int, rng: random.Random | None = None,
                   exclude: set[str] | None = None) -> list[Question]:
        """按官方领域权重抽题，模拟真实考卷的领域分布。"""
        rng = rng or random.Random()
        exclude = exclude or set()
        picked: list[Question] = []

        for domain, meta in DOMAINS.items():
            pool = [q for q in self.by_domain(domain) if q.id not in exclude]
            want = round(count * meta["weight"])
            rng.shuffle(pool)
            picked.extend(pool[:want])

        # 权重取整后可能多退少补，用剩余题目补齐
        if len(picked) < count:
            chosen = {q.id for q in picked}
            rest = [q for q in self.questions
                    if q.id not in chosen and q.id not in exclude]
            rng.shuffle(rest)
            picked.extend(rest[: count - len(picked)])
        picked = picked[:count]
        rng.shuffle(picked)
        return picked
