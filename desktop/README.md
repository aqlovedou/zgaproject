# AZ-900 刷题台（桌面版）

Vue 3 + TypeScript 前端，Rust + SQLite 后端，Tauri 打包成桌面应用。
成绩和错题本存在本地 SQLite 文件里，可以直接备份、拷走、用任何 SQLite 工具打开。

## 跑起来

需要先装好 **Node 18+** 和 **Rust**（https://rustup.rs）。

```bash
cd desktop
npm install
npm run dev        # 开发模式，热重载
npm run build      # 打包成桌面安装包
```

各平台还需要 Tauri 的系统依赖，官方清单：https://tauri.app/start/prerequisites/

- **Windows**：Microsoft C++ 生成工具 + WebView2（Win11 自带）
- **macOS**：`xcode-select --install`
- **Linux**：`libwebkit2gtk-4.1-dev libgtk-3-dev libayatana-appindicator3-dev librsvg2-dev patchelf`

`npm run build` 的产物：

| 平台 | 产物 |
| --- | --- |
| Windows | `target/release/bundle/msi/*.msi` 或 `nsis/*.exe` |
| macOS | `target/release/bundle/dmg/*.dmg` |
| Linux | `target/release/bundle/deb/*.deb`、`appimage/*.AppImage` |

（这是 Cargo 工作区，所以产物在 `desktop/target/` 而不是 `desktop/src-tauri/target/`。）

> 交叉编译很麻烦：要出 Windows 安装包就在 Windows 上构建，macOS 同理。

## 数据存在哪

SQLite 文件在系统的应用数据目录，应用底部会显示完整路径：

| 平台 | 路径 |
| --- | --- |
| Windows | `%APPDATA%\com.az900.examprep\az900.db` |
| macOS | `~/Library/Application Support/com.az900.examprep/az900.db` |
| Linux | `~/.local/share/com.az900.examprep/az900.db` |

备份就是拷走这一个文件。卸载重装不会丢成绩（除非手动删掉这个目录）。

## 结构

```
desktop/
├─ crates/core/            Rust 核心层（不依赖 Tauri，可单独测试）
│  ├─ src/model.rs         领域模型：Question / Session / WrongEntry
│  ├─ src/scoring.rs       判分、1000 分制折算、领域拆分
│  ├─ src/leitner.rs       错题本的间隔重复算法
│  ├─ src/exam.rs          按官方领域权重组卷
│  ├─ src/schema.sql       建表语句
│  ├─ src/db.rs            SQLite 数据层，所有 SQL 集中在此
│  └─ tests/db_flow.rs     端到端测试：用真实题库跑完整流程
├─ src-tauri/              Tauri 外壳（很薄）
│  ├─ src/commands.rs      前端 invoke 的全部入口
│  ├─ src/state.rs         数据库连接 + 题库缓存
│  ├─ capabilities/        窗口权限（打开外部链接需要显式声明）
│  └─ tauri.conf.json      窗口、打包、CSP 配置
└─ src/                    Vue 3 + TypeScript 前端
   ├─ types.ts             与 Rust serde 输出一一对应的类型
   ├─ api.ts               带类型的命令封装，前端只通过这里和 Rust 说话
   ├─ format.ts            倒计时、正确率、复习时间等格式化
   ├─ stores/quiz.ts       当前这套题的状态和计时器
   ├─ views/               计划 / 仪表盘 / 专项练习 / 答题 / 成绩单 / 错题本
   └─ components/          题卡、讲评、三个图表组件
```

### 为什么分成两个 Rust crate

`crates/core` 不依赖 Tauri，所以判分、组卷、间隔重复、SQLite 读写这些真正有逻辑的部分
可以用普通 `cargo test` 完整测试，不需要启动图形界面。`src-tauri` 只负责把这些能力
包装成命令暴露给前端。

## 测试

```bash
cargo test              # Rust：29 项（16 单元 + 13 端到端）
npm test                # 前端：19 项
npm run typecheck       # vue-tsc 严格模式
```

Rust 端到端测试用的是仓库里真实的 179 道题，覆盖：
题库导入与校验（每个错误选项都必须写明为什么错、每题都必须有官方出处）、
考试到交卷、错题进错题本、复习到攻克、再答错打回、统计聚合、SQLite 落盘后重开。

## 两个设计决定

**成绩不信任前端。** 交卷时后端从 `answer` 表重新统计答对数并折算分数，
不采用前端传来的分数。前端传的只有「哪道题、选了什么、花了多少秒」。

**统计不存缓存表。** 成绩趋势、领域正确率、覆盖率全部从 `answer` 实时聚合。
代价是查询多跑几毫秒（这个数据量下无感），好处是永远不会出现
「统计数字和明细对不上」——这类 bug 一旦出现，你会开始怀疑所有数据。

## 题库

题目源文件在仓库根目录的 `data/questions/*.json`，编译时通过 `include_str!`
嵌进二进制，发布版不需要外部文件。改完题库重新 `npm run build` 即可，
同一个题号是覆盖而不是新增。

题目依据微软公开的 AZ-900 技能大纲原创编写，不含任何考试泄题内容。
