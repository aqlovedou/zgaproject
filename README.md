# AZ-900 五天冲刺刷题系统

面向 Microsoft Azure Fundamentals（AZ-900）考试的命令行刷题程序：按官方领域权重组卷的全真模拟考、
即时讲评的专项练习、带间隔重复算法的错题本，以及达标判断。

**179 道题**，每题都包含：详细解析 + 每个错误选项为什么错 + Microsoft Learn 官方文档出处。

## 关于题目来源（重要）

本题库是**依据微软公开的 AZ-900 官方技能大纲（Skills Outline）原创编写**的练习题，
不是、也不会包含真实考试的泄题内容（那违反考试保密协议且答案往往是错的）。

AZ-900 的考点范围完全由官方大纲界定，题型高度固定（单选、多选、判断）。本题库按大纲的
三大领域与官方权重覆盖全部考点，并为每题标注官方文档出处，便于回溯原文。

## 快速开始

```bash
git clone <this-repo> && cd zgaproject
python3 az900.py plan          # 看 5 天冲刺计划
python3 az900.py exam          # 50 题 / 45 分钟全真模拟考
```

只需要 Python 3.10+，无任何第三方依赖。

## 命令

| 命令 | 作用 |
| --- | --- |
| `python3 az900.py plan [--day N]` | 5 天冲刺计划，每天的考点、任务和达标线 |
| `python3 az900.py exam [--count 50] [--time 45]` | 全真模拟考：按官方权重组卷、限时、考完统一讲评 |
| `python3 az900.py practice --domain 1｜2｜3` | 按领域专项练习，答一题讲一题 |
| `python3 az900.py practice --topic 存储` | 按考点练习（考点名见 `topics`） |
| `python3 az900.py practice --keyword ExpressRoute` | 按关键词搜题练习 |
| `python3 az900.py wrong` | 查看错题本 |
| `python3 az900.py wrong --review` | 错题复习（间隔重复，答对才会推迟下次出现） |
| `python3 az900.py stats` | 学习进度、各领域正确率、模拟考成绩趋势、达标判断 |
| `python3 az900.py topics` | 列出全部考点及题量 |
| `python3 az900.py show D2-042` | 查看某道题的完整解析 |

答题时输入选项字母（多选题连写，如 `AC`），`s` 跳过，`q` 退出。

## 评分与达标标准

- 折算为官方的 **1000 分制，700 分及格**
- 全真模拟按官方领域权重组卷：领域1 约 27.5%、领域2 约 37.5%、领域3 约 32.5%
- 成绩单给出各领域正确率和「最该补的考点」
- `stats` 的达标判断：**连续两次模拟 ≥ 800 分**，且三个领域正确率均 ≥ 80%

## 错题本怎么工作

答错的题自动收录，记录你选了什么、正确答案、错误原因和官方出处。

复习采用 Leitner 盒子间隔重复：每题有 1–5 级掌握度，答对升一级并推迟下次出现
（3 小时 → 12 小时 → 1 天 → 2 天），答错立即打回 1 级。
**掌握度达到 4 级且连续答对 2 次**才算攻克，从待复习列表中移除。

学习数据保存在 `data/state/`（已在 .gitignore 中，不会提交）：
- `progress.json` — 每次考试和练习的完整作答记录
- `wrongbook.json` — 错题本

## 考点覆盖（按官方大纲）

**领域 1 · 云概念（官方 25–30%）** — 47 题
共享责任模型、云部署模型（公有/私有/混合/多云、Azure Arc、Azure VMware 解决方案）、
消费模型与 CapEx/OpEx、云的优势（高可用、缩放、弹性、可靠性、可预测性、治理、安全、管理性）、
IaaS/PaaS/SaaS。

**领域 2 · Azure 架构与服务（官方 35–40%）** — 72 题
区域与区域对、主权区域、可用性区域、资源组/订阅/管理组层级、ARM；
虚拟机与规模集、可用性集、Azure 虚拟桌面、容器（ACI / Container Apps / AKS）、Functions、App Service；
虚拟网络与子网、NSG、对等互连、VPN 网关、ExpressRoute、DNS、专用终结点、负载均衡；
存储账户与冗余（LRS/ZRS/GRS/GZRS/RA-*）、存储服务与访问层、Azure Migrate / Data Box / AzCopy / 文件同步；
Microsoft Entra ID 与域服务、MFA 与无密码、条件访问、SSO、RBAC、托管标识、
零信任、纵深防御、Defender for Cloud、Key Vault。

**领域 3 · Azure 管理与治理（官方 30–35%）** — 60 题
成本影响因素、预留实例/Spot/混合权益、定价计算器 vs TCO 计算器、Cost Management 与预算、标记；
Azure Policy（效果、计划、修正）、资源锁、Microsoft Purview、服务信任门户、法规合规仪表板；
门户 / CLI / PowerShell / Cloud Shell / 移动应用、ARM 模板与 Bicep、Azure Arc；
Azure Monitor（指标与日志、Log Analytics 与 KQL、警报与操作组）、Application Insights、
Azure Advisor、Service Health 与资源运行状况。

## 网页版

`web/` 是同一套题库的网页版（手机/电脑都能刷，进度和错题本会保存）：

```bash
python3 tools/build_web.py     # 从 data/questions/ 重新打包 web/bank.js
```

改了题库之后跑一次这个脚本，网页版就同步了。`web/index.html` 以 Claude Artifact 形式发布，
运行时用 Artifact 的 db 能力保存进度；本地直接用浏览器打开时会退回到浏览器本地存储。

## 开发

```bash
python3 -m pytest tests/ -q
```

测试覆盖题库完整性（每个错误选项必须有解释、每题必须有 learn.microsoft.com 出处、
题号与领域一致）、组卷的领域权重、评分折算、选项解析和错题本的间隔重复逻辑。

题库是 `data/questions/*.json`，schema 见 `az900/models.py` 的 `Question`。
新增题目只需往 JSON 里加，`Bank.load()` 会自动校验格式。
