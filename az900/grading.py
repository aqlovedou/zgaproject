"""评分：折算成 Azure 认证的 1000 分制，并给出领域得分。"""
from __future__ import annotations

from typing import Any, Iterable

from .models import DOMAINS, MAX_SCORE, PASS_SCORE, Answered, Question


def scaled_score(correct: int, total: int) -> int:
    if total == 0:
        return 0
    return round(correct / total * MAX_SCORE)


def passed(score: int) -> bool:
    return score >= PASS_SCORE


def domain_breakdown(questions: Iterable[Question],
                     answers: dict[str, Answered]) -> dict[str, dict[str, Any]]:
    stats: dict[str, dict[str, Any]] = {
        key: {"total": 0, "correct": 0, "name_cn": meta["name_cn"],
              "no": meta["no"], "weight": meta["weight"]}
        for key, meta in DOMAINS.items()
    }
    for q in questions:
        bucket = stats[q.domain]
        bucket["total"] += 1
        got = answers.get(q.id)
        if got and got.correct:
            bucket["correct"] += 1
    for bucket in stats.values():
        bucket["rate"] = (bucket["correct"] / bucket["total"]) if bucket["total"] else 0.0
    return stats


def topic_weakness(questions: Iterable[Question],
                   answers: dict[str, Answered]) -> list[tuple[str, int, int]]:
    """返回 (考点, 错题数, 总题数)，按错得最多排序。"""
    agg: dict[str, list[int]] = {}
    for q in questions:
        row = agg.setdefault(q.topic, [0, 0])
        row[1] += 1
        got = answers.get(q.id)
        if got and not got.correct:
            row[0] += 1
    rows = [(topic, wrong, total) for topic, (wrong, total) in agg.items() if wrong]
    return sorted(rows, key=lambda r: (-r[1], -r[2]))
