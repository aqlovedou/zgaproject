"""5 天冲刺计划：每天的考点、任务和达标线。"""
from __future__ import annotations

from typing import Any

from . import ui

PLAN: list[dict[str, Any]] = [
    {
        "day": 1,
        "title": "云概念打底 + Azure 架构骨架",
        "focus": ["共享责任模型", "云部署模型", "消费模型与成本", "云的优势", "IaaS/PaaS/SaaS",
                  "区域与可用性区域", "资源组织层级"],
        "tasks": [
            "python3 az900.py practice --domain 1            # 领域1 全刷一遍",
            "python3 az900.py practice --topic 区域 --count 12",
            "python3 az900.py practice --topic 资源组织 --count 10",
            "python3 az900.py exam --count 20                # 20 题小测热身",
        ],
        "target": "领域1 正确率 ≥ 80%",
    },
    {
        "day": 2,
        "title": "计算 + 网络（领域2 的最大失分区）",
        "focus": ["虚拟机与规模集", "容器与 AKS", "Azure Functions", "App Service",
                  "虚拟网络与子网", "VPN 网关与 ExpressRoute", "公共/专用终结点"],
        "tasks": [
            "python3 az900.py practice --topic 计算 --count 18",
            "python3 az900.py practice --topic 网络 --count 18",
            "python3 az900.py wrong --review                  # 清昨天的错题",
            "python3 az900.py exam --count 30",
        ],
        "target": "领域2 计算/网络类题目正确率 ≥ 75%",
    },
    {
        "day": 3,
        "title": "存储 + 身份与安全",
        "focus": ["存储账户与冗余（LRS/ZRS/GRS/GZRS）", "存储服务与访问层", "迁移与数据搬运工具",
                  "Microsoft Entra ID", "MFA 与条件访问", "RBAC", "零信任与纵深防御",
                  "Microsoft Defender for Cloud"],
        "tasks": [
            "python3 az900.py practice --topic 存储 --count 18",
            "python3 az900.py practice --topic 身份 --count 12",
            "python3 az900.py practice --topic 安全 --count 12",
            "python3 az900.py wrong --review",
            "python3 az900.py exam --count 40",
        ],
        "target": "领域2 整体正确率 ≥ 75%，模拟考 ≥ 650 分",
    },
    {
        "day": 4,
        "title": "管理与治理 + 第一次全真模拟",
        "focus": ["成本管理与标记", "定价计算器 vs TCO 计算器", "Azure Policy 与资源锁",
                  "Microsoft Purview 与服务信任门户", "门户/CLI/PowerShell/Cloud Shell",
                  "ARM 与 Bicep", "Azure Arc", "Azure Monitor / Advisor / Service Health"],
        "tasks": [
            "python3 az900.py practice --domain 3            # 领域3 全刷一遍",
            "python3 az900.py wrong --review",
            "python3 az900.py exam                            # 50 题 45 分钟全真模拟",
        ],
        "target": "全真模拟 ≥ 700 分（首次达标）",
    },
    {
        "day": 5,
        "title": "错题清零 + 两次全真模拟稳定达标",
        "focus": ["错题本全部攻克", "三大领域均衡", "考试节奏（平均每题 ≤ 54 秒）"],
        "tasks": [
            "python3 az900.py wrong --review                  # 反复刷到错题本清空",
            "python3 az900.py exam                            # 第二次全真模拟",
            "python3 az900.py wrong --review",
            "python3 az900.py exam                            # 第三次全真模拟",
            "python3 az900.py stats                           # 确认三领域都 ≥ 80%",
        ],
        "target": "连续两次模拟 ≥ 800 分，且错题本已攻克数 ≥ 90%",
    },
]


def render_day(day: int) -> str:
    entry = next((d for d in PLAN if d["day"] == day), None)
    if entry is None:
        return ui.red(f"没有第 {day} 天的计划（计划为 1-5 天）。")
    lines = [ui.title(f"Day {entry['day']}：{entry['title']}")]
    lines.append(ui.bold("今日考点"))
    for f in entry["focus"]:
        lines.append(f"  • {f}")
    lines.append("")
    lines.append(ui.bold("今日任务"))
    for t in entry["tasks"]:
        lines.append(f"  $ {ui.blue(t)}")
    lines.append("")
    lines.append(f"{ui.bold('达标线')} {ui.yellow(entry['target'])}")
    return "\n".join(lines)


def render_all() -> str:
    lines = [ui.title("AZ-900 五天冲刺计划")]
    for entry in PLAN:
        lines.append(f"{ui.bold('Day ' + str(entry['day']))} {entry['title']}")
        lines.append(f"  考点：{'、'.join(entry['focus'][:4])} …")
        lines.append(f"  达标：{ui.yellow(entry['target'])}")
        lines.append("")
    lines.append(ui.dim("查看某天详情： python3 az900.py plan --day 2"))
    return "\n".join(lines)
