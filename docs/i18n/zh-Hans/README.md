<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | **简体中文** | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### 专属于你的 AI 智能体

*不是通用模板，而是根据你的角色、专业和目标编译出的定制化工作台。*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

大多数 AI 工具给你一套固定功能，然后说"自己摸索吧"。BYOH 反其道而行：通过一次访谈了解你真正的工作方式，然后生成专属的智能体工作台——技能、记忆、流水线——开箱即用，无需手动调教。

## 适合哪些人

- **开发者** —— 想要一个已经了解自己技术栈、测试风格和交付节奏的智能体
- **研究者** —— 需要将文献检索、引用追踪和综合分析串联在一起的完整流水线
- **创作者** —— 想要一个匹配自己写作风格和项目结构的创作伙伴
- **业务分析师** —— 需要决策框架和报告流水线，而不是裸聊天

如果你曾想过"要是 AI 真的了解我的上下文就好了"——这正是 BYOH 要解决的问题。

## 60 秒上手

BYOH 从一开始就是为 AI 智能体驱动而设计的。安装它，通过 MCP 连接你的主机，然后直接对话——对话本身就是访谈、向导和构建过程。

```
1. Install byoh              # 一行安装（见下文）
2. Connect your host via MCP # byoh serve —— 任何兼容 MCP 的智能体
3. "Build me a harness"      # 你的智能体扫描仓库并编译出结果
```

下次会话时，你的主机会自动加载这套工作台——智能体、技能、记忆和流水线全部按你的工作方式调校到位。

**更喜欢终端？** 同样的流程用 CLI 完成：
```
byoh profile init me        # 扫描项目（只读，不修改任何文件）
byoh profile interview me   # 关于你的角色和目标的简短对话
byoh compile me             # 生成专属工作台
byoh install me             # 部署到 Claude / Codex / agy
```

**已经知道自己需要什么？** 直接浏览社区目录：
```bash
byoh catalog index                                 # 拉取 Top 100 插件列表（几秒钟）
byoh catalog search "code review"                  # 搜索相关插件
byoh catalog vendor anthropics/claude-code-review  # 添加一个到工作台
```

## 安装

### 二进制安装（推荐，无需 Rust 环境）

**macOS / Linux：**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows（PowerShell）：**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**从源码构建：**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # 验证安装
```

### 连接 AI 主机

BYOH 说 MCP 协议，因此任何兼容 MCP 的智能体都能驱动它。安装上面的二进制文件，启动服务器，你的主机就能直接调用所有 BYOH 工具：

```bash
byoh serve   # stdio MCP 服务器
```

对于**其他主机**（Cursor、Zed、Continue……），把 `byoh` 加到主机的 MCP 配置中：
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

使用的是 **Claude Code、Codex 或 agy**？改用插件安装——它打包了 MCP 服务器，并在首次加载时自动安装二进制（无需 Rust）：

**Claude Code：**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy（Antigravity）：**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex：**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

> **注意：** 仓库目前为私有。请使用上面的路径。公开后将出现在 `epicsagas/plugins` 共享市场中。

## 智能体主导模式

主机连接好之后，你不需要输命令——只需开口对话。你的智能体会直接调用 BYOH 的 14 个工具，对话本身就是访谈、构建和进化循环：

可用工具：`profile_create`、`profile_scan`、`profile_interview`、`profile_confirm`、`compile`、`evolve_cycle`、`genre_list`、`registry_clone_skill`、`catalog_search`、`catalog_vendor` 等。

## 用 CLI 构建你的第一个工作台

同样的步骤，从终端驱动：

### 第一步 —— 创建档案
```bash
byoh profile init me --paths ./src ./docs   # 自动扫描项目结构
byoh profile interview me                   # 约 5 分钟的引导式问答
byoh profile confirm me --genre developer   # 确认并锁定角色
```

BYOH 会询问你的职责、专业水平、使用工具和 30 天目标。问卷自动适配——研究者和开发者收到的问题不同。

### 第二步 —— 编译并安装
```bash
byoh compile me          # 验证并生成 HarnessBundle（带门控）
byoh render me --target claude   # 或：codex | agy | all
byoh install me          # 安全部署到 dist/
```

### 第三步 —— 运行与进化
```bash
byoh run me              # 以你的工作台启动
byoh evolve me           # 根据会话反馈改进工作台
```

`evolve` 执行三重门控循环（Critic / Seesaw / Stagnation），无法绕过——进化过程安全且可审计。

## 插件目录

目录提供按 Stars 排序的 Top 100 Claude 插件（每日自动更新），让你在终端里就能发现并添加社区技能。

```bash
# 一次性索引——几秒钟下载预构建包
byoh catalog index

# 索引后完全离线搜索，无需网络
byoh catalog search "memory" --genre developer --limit 5

# 将插件添加到工作台
# 自动从克隆仓库中提取 license、keywords 和 genre
byoh catalog vendor obra/superpowers --genre developer
```

LLM 智能体（通过 `catalog_search` / `catalog_vendor` MCP 工具）可以自主完成整个流程——你也可以直接用 CLI 驱动。

## 完整 CLI 参考

```bash
# 档案管理
byoh profile init <slug> [--paths ...]       # 非破坏性项目扫描
byoh profile interview <slug>                # 引导式问答
byoh profile confirm <slug> --genre <g>      # 确认并锁定档案

# 构建
byoh compile <slug> [--dry-run]              # 验证 + 生成 HarnessBundle
byoh render <slug> --target <host>           # claude | codex | agy | all
byoh install <slug> [--host <dir>]           # 部署到 dist/ 或实际 plugin 目录

# 运行与进化
byoh run <slug>
byoh evolve <slug>

# 社区技能
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# 目录
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## 底层原理

BYOH 的合成引擎将你的档案标签与技能注册表匹配，按依赖关系排序成流水线，输出 `HarnessBundle`——一个可以渲染为任意支持主机原生格式的 git 制品。

- **四环安全模型** —— 从内置技能（Ring 1）到社区/未信任技能（Ring 4），逐级加强验证
- **三重门控进化** —— 每次 `evolve` 必须通过 Critic（质量）、Seesaw（回归）、Stagnation（平台期）三道关卡，无法绕过
- **目标导向流水线** —— 声明 30 天目标（产品发布、研究报告、安全交付……）后自动叠加对应技能阶梯

架构：六边形架构——`domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`。完整架构指南见 `AGENTS.md`。

## 构建与开发

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # 单元测试 + 端到端测试
cvp                               # 并行执行：check → clippy → test → fmt → build
```

`mcp` feature（stdio MCP 服务器）默认开启。BYOH 本身不自带检索后端——如需检索能力，请把生成的工作台指向一个文档服务器，例如 [alcove](https://github.com/epicsagas/alcove)。

## 许可证

Apache-2.0.
