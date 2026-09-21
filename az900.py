#!/usr/bin/env python3
"""AZ-900 备考刷题系统入口。

用法：
    python3 az900.py plan              # 看 5 天冲刺计划
    python3 az900.py exam              # 全真模拟考
    python3 az900.py practice --domain 1
    python3 az900.py wrong --review    # 错题本复习
    python3 az900.py stats             # 进度与达标判断
"""
import sys

from az900.cli import main

if __name__ == "__main__":
    sys.exit(main())
