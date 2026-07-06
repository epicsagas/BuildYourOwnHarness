> 本文是 [README.md](../../../README.md) 的简体中文译文。英文版本为权威原文。

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | [日本語](../ja/README.md) | **简体中文** | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### 专为你打造的 AI agent

*不是通用模板——而是根据你的角色、专长和目标编译出的 harness。*

<img src="assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

大多数 AI 配置只是塞给你一套固定工具，再说一句"祝你好运"。BYOH 反其道而行：它先访谈你，了解你实际在做什么，再生成一个个性化的 agent harness——skills、agents、goal pipelines——开箱即用，贴合你的工作流。

## 它面向谁？

- **开发者**：希望 agent 已经熟悉自己的技术栈、测试风格和交付节奏
- **研究者**：需要把文献综述、引用追踪和综合写作串到一起
- **创作者**：想要一个契合自己文风和项目结构的写作搭档
- **业务分析师**：需要决策框架和汇报流水线，而不是单纯聊天

如果你曾想过"真希望我的 AI 能真正懂我的上下文"——这正是 BYOH 做的事。

## 60 秒看懂它怎么工作

BYOH 的设计是由你的 AI agent 来驱动——而不是靠你敲命令。装好 plugin，然后直接开口聊。对话本身就是访谈、向导和构建全过程。

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # your agent scans your repo and compiles the result
```

下一次会话时，你的 host 会自动加载 harness——agents、skills 和 goal pipelines 都已为你调好。

## 安装 plugin（推荐）

使用 **Claude Code、Codex 或 agy**？直接安装 plugin。它打包了 MCP server，并**在首次加载时自动安装 binary**——无需 Rust 工具链，无需手动配置：

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy (Antigravity):**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

### 使用其他兼容 MCP 的 host？

BYOH 说 MCP，所以 Cursor、Zed、Continue 等也能用。先装一次 [binary](#installation)，再把 host 指向这个 server：

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **注：** 该仓库目前为私有。请使用上述路径。公开后它会出现在共享的 `epicsagas/plugins` marketplace 中。

## Agent 主导模式——主路径

host 连上之后，你不需要敲命令——直接聊天就行。你的 agent 会直接调用 BYOH 的 MCP tools，而对话本身就是访谈、构建和 evolve 循环：

> **你：** *我是一名后端 Go 开发者，本月要交付一个 payments API。帮我构建一个 harness。*
>
> **Agent：** *（通过 `profile_scan` 扫描你的仓库，再用 `profile_interview` 问几个有针对性的问题，将 genre 锁定为 `developer`）* → 编译出一个 `HarnessBundle` → 把 agents、skills 和一条 secure-ship goal pipeline 装进 Claude Code。搞定——下一次会话时，你的 agent 已经会说你的技术栈了。

同一套流程，按建议的 tool 顺序：

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → install_plugin
```

`build` 会合成 bundle（compile + preset 注入 + static gate），并将每个 skill 分类为 `matched`（注入了真实 preset 内容）或 `skeleton`（仍是 genre 模板占位符）——agent 据此判断是立即安装还是先迭代 profile。

可用 tools：`profile_read`、`profile_create`、`profile_scan`、`profile_interview`、`profile_confirm`、`build`、`render_plugin`、`install_plugin`、`catalog_search`、`catalog_vendor`。

想让 agent 带你走完整套流程？只要说一句 *"build my harness"*——内置的 `byoh-guide` agent 会编排整个过程。

## Plugin 目录

该目录基于 [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) 的 README 构建——那是一份由社区维护、按 star 排名、涵盖 Top 100 Claude plugin 仓库的清单。BYOH 内置一份预构建 bundle（**每周**重建，周一 03:17 UTC），所以 `byoh catalog index` 几秒内就能完成；传入 `--no-bundle` 可直接解析上游清单。

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

LLM agent（通过 `catalog_search` / `catalog_vendor` MCP tools）可以自主完成整套流程——*"给我的 harness 加一个 memory plugin"*——你也可以直接在 CLI 上手动操作。

少数 companion tools 会作为**参考资料**（而非依赖）注入到搜索结果中：BYOH 自身的执行层工具——[alcove](https://github.com/epicsagas/alcove)（doc server）、[obsidian-forge](https://github.com/epicsagas/obsidian-forge)（vault 自动化）、[epic-harness](https://github.com/epicsagas/epic-harness)（hook/skill runtime）——会在相关场景中浮现（一次"doc server" / "search backend"查询会找到 alcove），便于 agent 在合适时机推荐。只有你确实需要时才去 vendor 它；无论怎样，bundle 始终不携带依赖。

## 进阶用户：CLI（可选）

上面每一条流程也都能在终端上完成。CLI 是**辅助性**的——适合脚本、CI，或你不想聊天的时候——但 agent 主导的路径才是设计初衷。

### 你的第一个 harness——从 CLI 开始

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # 合成（compile + preset 注入 + static gate）并写出 HarnessBundle; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

访谈本身由 agent 主导（`profile_interview` MCP tool）——对话即访谈，因此不存在交互式 CLI 访谈。build 的 static gate（Critic / Seesaw / Stagnation 安全门齐备性、MCP schema、hook 输入）始终运行且永远无法绕过——因此 bundle 在发布前保证结构有效。安装后的改进是后续会话中的对话式复盘，而不是一次工具调用。

## 底层如何工作

BYOH 的 synthesis engine 把你的 profile 标签匹配到 skill registry，按依赖关系排成一条 pipeline，并输出一个 `HarnessBundle`——一个 git 就绪的产物，可渲染为任意受支持 host 的原生格式。

- **4-ring 安全模型**——从 lifecycle spec（Ring 0）和内置 pipeline skills（Ring 1）一直到 community/untrusted skills（Ring 3），每一层都配有递进的校验；vendored skills 以 sha256 固定，并在读取 + embed 时校验
- **3-gate 安全基线**——每次 build 的 static gate 都会确认 Critic（质量）、Seesaw（回归）和 Stagnation（停滞）三道闸门齐备；无法绕过
- **面向目标的 pipelines**——声明一个 30 天目标（产品上线、研究报告、安全交付……）会自动叠加一条匹配的 skill ladder

架构：六边形——`domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`。完整指南见 `AGENTS.md`。

## 完整 CLI 参考

CLI 刻意保持精简：机器入口（`serve`、CI 中的 `catalog index`、面向维护者的 `vendor`）加上核心构建流程的可脚本化镜像。Interview 和 evolution 仅限 MCP（agent 主导）。

```bash
# Profile
byoh profile init <slug> [--paths ...]      # non-destructive project scan
byoh profile confirm <slug> --genre <g> [--goal <text>]  # confirm and lock profile
byoh profile show <slug>                    # print the profile YAML

# Build (static gate always runs; render synthesizes: compile + preset injection)
byoh render <slug> [--target <host>]        # claude | codex | agy | all (default: all); writes the HarnessBundle
byoh install <slug> [--target <host>] [--scope local|global|publish] [--host] [--force]  # dist/ tree; --scope decides where it goes (local=this project, global=HOME, publish=+LICENSE/.gitignore+git steps). --host is legacy for --scope global.

# Community skills (maintainer/build-time; sha256-pinned and verified at read + embed time)
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# Catalog
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# Diagnostics / server
byoh doctor                                 # check execution-layer tools
byoh serve                                  # stdio MCP server (agent-led mode)
```

profile 和 catalog 缓存默认存放在 `~/.byoh` 下（可用 `BYOH_HOME` 覆盖）。

## 安装

仅当你**不**使用 plugin（plugin 会自动安装 binary），或想把 BYOH 接到非 plugin 的 MCP host 上时才需要。

### Binary（无需 Rust 工具链）

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**从源码构建：**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## 构建与开发

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

`mcp` feature（stdio MCP server）默认开启。BYOH 不内嵌任何知识库——若需检索能力，请把你生成的 harness 指向一个 doc server，例如 [alcove](https://github.com/epicsagas/alcove)。

## 致谢

BYOH 站在若干社区项目的肩膀上：

- **Plugin 目录**——源自 [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins)，一份按 star 排名、涵盖 Top 100 Claude plugin 仓库的社区清单。没有它就没有这个目录。
- **Companion tools**——设计与 [alcove](https://github.com/epicsagas/alcove)（doc server / RAG）、[Episteme](https://github.com/epicsagas/Episteme)（knowledge graph）以及 [obsidian-forge](https://github.com/epicsagas/obsidian-forge)（vault 自动化）互通互联。
- **开源技术栈**——构建于 [clap](https://docs.rs/clap)、[serde](https://serde.rs)、[ureq](https://docs.rs/ureq) 以及 Rust 生态之上。

Catalog 条目和 vendored community skills 保留各自原有的 license（在 vendor 时自动检测）。BYOH 本身采用 Apache-2.0。

## License

Apache-2.0。
