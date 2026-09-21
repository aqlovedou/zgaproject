"""AZ-900 题库与作答记录的数据模型。"""
from __future__ import annotations

import time
from dataclasses import dataclass, field
from typing import Any

# 官方技能大纲的三大领域与权重（Microsoft AZ-900 Study Guide）
DOMAINS: dict[str, dict[str, Any]] = {
    "cloud-concepts": {
        "no": 1,
        "name_cn": "云概念",
        "name_en": "Describe cloud concepts",
        "weight": 0.275,  # 官方 25-30%
    },
    "architecture-services": {
        "no": 2,
        "name_cn": "Azure 架构与服务",
        "name_en": "Describe Azure architecture and services",
        "weight": 0.375,  # 官方 35-40%
    },
    "management-governance": {
        "no": 3,
        "name_cn": "Azure 管理与治理",
        "name_en": "Describe Azure management and governance",
        "weight": 0.325,  # 官方 30-35%
    },
}

DOMAIN_BY_NO = {d["no"]: key for key, d in DOMAINS.items()}

# 考试常量
PASS_SCORE = 700          # 满分 1000，700 分及格
MAX_SCORE = 1000
DEFAULT_EXAM_COUNT = 50   # 真实考试约 40-60 题
DEFAULT_EXAM_MINUTES = 45 # 答题时间 45 分钟（总座位时间约 65 分钟）

TYPE_LABEL = {
    "single": "单选题",
    "multi": "多选题",
    "truefalse": "判断题",
}


@dataclass(frozen=True)
class Question:
    id: str
    domain: str
    topic: str
    type: str
    question: str
    options: dict[str, str]
    answer: list[str]
    explanation: str
    reference_title: str
    reference_url: str
    option_reasons: dict[str, str] = field(default_factory=dict)
    difficulty: int = 2
    keywords: list[str] = field(default_factory=list)

    @property
    def domain_no(self) -> int:
        return DOMAINS[self.domain]["no"]

    @property
    def domain_cn(self) -> str:
        return DOMAINS[self.domain]["name_cn"]

    @property
    def type_label(self) -> str:
        return TYPE_LABEL.get(self.type, self.type)

    @property
    def answer_text(self) -> str:
        return "、".join(f"{k}. {self.options[k]}" for k in self.answer)

    def is_correct(self, picked: list[str]) -> bool:
        return sorted(picked) == sorted(self.answer)

    def wrong_reason(self, key: str) -> str:
        """某个错误选项为什么错。"""
        return self.option_reasons.get(key, "该选项与题干要求的场景不符。")

    @classmethod
    def from_dict(cls, raw: dict[str, Any]) -> "Question":
        ref = raw.get("reference", {})
        return cls(
            id=raw["id"],
            domain=raw["domain"],
            topic=raw["topic"],
            type=raw["type"],
            question=raw["question"],
            options=raw["options"],
            answer=list(raw["answer"]),
            explanation=raw["explanation"],
            reference_title=ref.get("title", ""),
            reference_url=ref.get("url", ""),
            option_reasons=raw.get("option_reasons", {}),
            difficulty=raw.get("difficulty", 2),
            keywords=raw.get("keywords", []),
        )


@dataclass
class Answered:
    """一次作答记录。"""
    qid: str
    picked: list[str]
    correct: bool
    seconds: float
    at: float = field(default_factory=time.time)

    def to_dict(self) -> dict[str, Any]:
        return {
            "qid": self.qid,
            "picked": self.picked,
            "correct": self.correct,
            "seconds": round(self.seconds, 1),
            "at": self.at,
        }
