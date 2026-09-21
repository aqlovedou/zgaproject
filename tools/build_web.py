#!/usr/bin/env python3
"""把 data/questions/*.json 打包成网页版用的 web/bank.js。

网页版和命令行版共用同一份题库，改完题目跑一次这个脚本即可同步。
"""
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
SRC = ROOT / "data" / "questions"
DEST = ROOT / "web" / "bank.js"

# 网页端用短字段名，170KB 左右，首屏加载快
FIELD_MAP = [
    ("id", "id"), ("domain", "d"), ("topic", "t"), ("type", "k"),
    ("question", "q"), ("options", "o"), ("answer", "a"),
    ("explanation", "e"), ("option_reasons", "r"),
]


def main() -> None:
    rows = []
    for path in sorted(SRC.glob("*.json")):
        for q in json.loads(path.read_text(encoding="utf-8"))["questions"]:
            row = {short: q[long] for long, short in FIELD_MAP}
            row["rt"] = q["reference"]["title"]
            row["ru"] = q["reference"]["url"]
            rows.append(row)

    DEST.parent.mkdir(exist_ok=True)
    payload = json.dumps(rows, ensure_ascii=False, separators=(",", ":"))
    DEST.write_text(f"window.AZ900_BANK={payload};", encoding="utf-8")
    print(f"{len(rows)} 题 -> {DEST.relative_to(ROOT)} ({DEST.stat().st_size // 1024} KB)")


if __name__ == "__main__":
    main()
