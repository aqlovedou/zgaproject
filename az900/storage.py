"""学习进度与错题本的本地存储（纯 JSON，无外部依赖）。"""
from __future__ import annotations

import json
import time
from pathlib import Path
from typing import Any

STATE_DIR = Path(__file__).resolve().parent.parent / "data" / "state"
PROGRESS_FILE = "progress.json"
WRONGBOOK_FILE = "wrongbook.json"


class Store:
    def __init__(self, state_dir: Path | None = None):
        self.dir = state_dir or STATE_DIR
        self.dir.mkdir(parents=True, exist_ok=True)
        self.progress = self._read(PROGRESS_FILE, {"exams": [], "practices": []})
        self.wrongbook = self._read(WRONGBOOK_FILE, {"entries": {}})

    # ---------- 文件读写 ----------
    def _path(self, name: str) -> Path:
        return self.dir / name

    def _read(self, name: str, default: dict[str, Any]) -> dict[str, Any]:
        path = self._path(name)
        if not path.exists():
            return default
        try:
            return json.loads(path.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            backup = path.with_suffix(".corrupt.json")
            path.rename(backup)
            return default

    def save(self) -> None:
        self._write(PROGRESS_FILE, self.progress)
        self._write(WRONGBOOK_FILE, self.wrongbook)

    def _write(self, name: str, payload: dict[str, Any]) -> None:
        path = self._path(name)
        tmp = path.with_suffix(".tmp")
        tmp.write_text(json.dumps(payload, ensure_ascii=False, indent=2), encoding="utf-8")
        tmp.replace(path)

    # ---------- 会话记录 ----------
    def record_session(self, kind: str, payload: dict[str, Any]) -> None:
        bucket = "exams" if kind == "exam" else "practices"
        self.progress.setdefault(bucket, []).append(payload)

    # ---------- 错题本（Leitner 盒子间隔复习）----------
    # box 越高表示掌握越牢，复习间隔越长；连续答对 2 次且 box>=4 视为已攻克
    BOX_INTERVALS_HOURS = {1: 0, 2: 3, 3: 12, 4: 24, 5: 48}

    def add_wrong(self, qid: str, picked: list[str], correct_answer: list[str],
                  domain: str, topic: str) -> None:
        entries = self.wrongbook.setdefault("entries", {})
        now = time.time()
        entry = entries.get(qid)
        if entry is None:
            entry = {
                "qid": qid,
                "domain": domain,
                "topic": topic,
                "wrong_count": 0,
                "right_streak": 0,
                "box": 1,
                "first_wrong_at": now,
                "history": [],
                "mastered": False,
            }
            entries[qid] = entry
        entry["wrong_count"] += 1
        entry["right_streak"] = 0
        entry["box"] = 1
        entry["mastered"] = False
        entry["last_wrong_at"] = now
        entry["last_picked"] = picked
        entry["correct_answer"] = correct_answer
        entry["due_at"] = now
        entry["history"].append({"at": now, "picked": picked, "correct": False})

    def mark_review_result(self, qid: str, correct: bool, picked: list[str]) -> None:
        entry = self.wrongbook.get("entries", {}).get(qid)
        if entry is None:
            return
        now = time.time()
        entry["history"].append({"at": now, "picked": picked, "correct": correct})
        if correct:
            entry["right_streak"] = entry.get("right_streak", 0) + 1
            entry["box"] = min(5, entry.get("box", 1) + 1)
            hours = self.BOX_INTERVALS_HOURS[entry["box"]]
            entry["due_at"] = now + hours * 3600
            if entry["box"] >= 4 and entry["right_streak"] >= 2:
                entry["mastered"] = True
        else:
            entry["wrong_count"] += 1
            entry["right_streak"] = 0
            entry["box"] = 1
            entry["mastered"] = False
            entry["due_at"] = now
            entry["last_wrong_at"] = now
            entry["last_picked"] = picked

    def wrong_entries(self, include_mastered: bool = False) -> list[dict[str, Any]]:
        entries = list(self.wrongbook.get("entries", {}).values())
        if not include_mastered:
            entries = [e for e in entries if not e.get("mastered")]
        return sorted(entries, key=lambda e: (-e.get("wrong_count", 0), e.get("due_at", 0)))

    def due_entries(self, now: float | None = None) -> list[dict[str, Any]]:
        now = now if now is not None else time.time()
        return [e for e in self.wrong_entries() if e.get("due_at", 0) <= now]
