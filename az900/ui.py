"""终端输出的小工具：颜色、分隔线、排版。"""
from __future__ import annotations

import os
import sys

_NO_COLOR = bool(os.environ.get("NO_COLOR")) or not sys.stdout.isatty()


def _c(code: str, text: str) -> str:
    return text if _NO_COLOR else f"\033[{code}m{text}\033[0m"


def bold(t: str) -> str:   return _c("1", t)
def red(t: str) -> str:    return _c("31", t)
def green(t: str) -> str:  return _c("32", t)
def yellow(t: str) -> str: return _c("33", t)
def blue(t: str) -> str:   return _c("36", t)
def dim(t: str) -> str:    return _c("2", t)


def rule(char: str = "─", width: int = 66) -> str:
    return dim(char * width)


def title(text: str) -> str:
    return f"\n{rule('═')}\n{bold(text)}\n{rule('═')}"


def bar(rate: float, width: int = 24) -> str:
    filled = int(round(rate * width))
    body = "█" * filled + "░" * (width - filled)
    color = green if rate >= 0.8 else (yellow if rate >= 0.6 else red)
    return color(body)
