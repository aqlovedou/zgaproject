# AZ-900 刷题台

面向 Microsoft Azure Fundamentals（AZ-900）考试的桌面刷题应用：按官方领域权重组卷的全真模拟考、
即时讲评的专项练习、带间隔重复算法的错题本，以及上考场准备度判断。

**250 道题**，依据微软公开的 AZ-900 技能大纲原创编写，覆盖三大领域的全部考点。
每题都包含：详细解析 + 每个错误选项为什么错 + Microsoft Learn 官方文档出处。

Vue 3 + TypeScript 前端，Rust + SQLite 后端，Tauri 打包成桌面应用。
成绩和错题本存在本地 SQLite 文件里，可以直接备份、拷走、用任何 SQLite 工具打开。

## 快速开始

需要先装好 **Node 18+** 和 **Rust**（https://rustup.rs）。

```bash
cd desktop
npm install
npm run dev        # 开发模式，热重载
npm run build      # 打包成 .msi / .dmg / .AppImage
```

各平台的系统依赖、产物路径、数据库位置和项目结构，详见 [`desktop/README.md`](desktop/README.md)。

## 关于题目来源

本题库**不含任何考试泄题内容**。微软的真题受保密协议保护，网上流传的题库答案错误率也很高。

AZ-900 的考点范围完全由官方公开的 Skills Outline 界定，题型高度固定（单选、多选、判断）。
本题库按大纲的三大领域与官方权重覆盖全部考点，并为每题标注官方文档出处，便于回溯原文。

官方考试页：https://learn.microsoft.com/credentials/certifications/azure-fundamentals/

## 题库

题目源文件在 `data/questions/*.json`，编译时通过 `include_str!` 嵌进二进制，
发布版不需要外部文件。改完题库重新 `npm run build` 即可，同一个题号是覆盖而不是新增。

| 领域 | 官方权重 | 本题库 | 覆盖考点 |
| --- | --- | --- | --- |
| 1 云概念 | 25–30% | 69 题 | 共享责任模型、云部署模型、消费模型与 CapEx/OpEx、云的优势、IaaS/PaaS/SaaS |
| 2 Azure 架构与服务 | 35–40% | 95 题 | 区域与可用性区域、资源组织层级、计算、网络、存储、身份与安全 |
| 3 Azure 管理与治理 | 30–35% | 86 题 | 成本管理与标记、Policy 与资源锁、Purview、部署工具、监控 |

题库格式的校验（每个错误选项必须写明为什么错、每题必须有 learn.microsoft.com 出处、
题号与领域一致）由 `desktop/crates/core/tests/db_flow.rs` 的端到端测试保证。

## 考试规则

满分 1000 分，**700 分及格**，约 40–60 题，答题时间 45 分钟（总座位时间约 65 分钟）。
应用的模拟考按官方权重组卷并限时，节奏与真实考试一致（平均每题 54 秒）。

## 测试

```bash
cd desktop
cargo test           # Rust：29 项（16 单元 + 13 端到端）
npm test             # 前端：19 项
npm run typecheck    # vue-tsc 严格模式
```
