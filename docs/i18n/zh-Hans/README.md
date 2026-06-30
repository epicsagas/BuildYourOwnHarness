> 本文档是 [README.md](../../README.md) 的简体中文翻译。英文版本为权威原始来源，可能更新。
>
> ⚠️ Auto-translation pending — the English source below awaits translation via the i18n workflow.

# BuildYourOwnHarness (BYOH)

> Interactively collect a user's tacit knowledge, data, business genre, and goals — then **generate, deploy, operate, and evolve a personalized AI agent harness**.

BYOH adds a **generation layer** on top of the validated building blocks of the [epiccounty](https://github.com/epicsagas) workspace. Instead of shipping a fixed skill/memory/pipeline set, it compiles a *unique* harness per user from an interview.

## What it does

A confirmed user profile (genre + expertise + 30-day goal) drives a synthesis engine that **recombines registry skills by keyword** into an ordered pipeline, producing a `HarnessBundle` that is *not* a fixed genre template. The whole pipeline is closed-loop and gated by three safety gates (Critic / Seesaw / Stagnation) that can never be bypassed.

```
profile (interview → genre → confirm) → compile → synthesize → render → install → run → evolve
```

## Build & verify

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                       # unit + e2e
./target/release/byoh --help
```

Hexagonal architecture: `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`.

## CLI

```bash
byoh profile init <slug> [--paths ...]   # S1 autoscan (non-destructive)
byoh profile interview <slug>            # S2 interview (Suggest + Council)
byoh profile confirm <slug> --genre <g>  # S3 wizard confirm
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>
byoh compile <slug> [--dry-run]          # static gate + dry-run gate → HarnessBundle
byoh render <slug> --target claude       # claude | codex | agy | all (git-ready)
byoh install <slug>                      # safe dist/ install (--host for live plugin dir)
byoh run <slug>
byoh evolve <slug>                       # 3门安全门进化循环
byoh catalog index [--limit N]           # 解析 quemsah 前 100 README → ~/.byoh/catalog.json
byoh catalog search "<查询>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### 代理驱动模式（MCP 服务器）

`byoh serve`（`--features mcp`）启动一个 stdio MCP 服务器，让 LLM 代理**驱动 BYOH** — CLI 退为次要（控制反转）。14 个工具（`profile_*`、`rag_*`、`genre_list`、`compile`、`evolve_cycle`、`registry_clone_skill`、`catalog_search`、`catalog_vendor`）可通过 `tools/list` 发现。对话本身即为采访/向导流程。

```bash
cargo build --release --features mcp
byoh serve
```

## 核心：合成、外部引入与插件目录

- **合成引擎** — `synthesize(profile)` 将注册表技能与 profile 标签匹配，按顺序组成流水线，并强制经过三门安全门重新验证（不可绕过）。当 30 天目标匹配时，目标导向流水线（product-launch / decision / research-report / secure-ship / …）会叠加技能阶梯和代理集合。
- **社区技能外部引入**（RFC M3）— `byoh vendor add` 获取外部 `SKILL.md`（本地路径或 git URL），执行静态校验 + sha256 哈希，并在构建时通过 `build.rs` 嵌入 **Ring 3**（限制最严）。外部技能作为不信任代码加入合成流程。
- **插件目录** — `byoh catalog index` 从精选的 [quemsah/awesome-claude-plugins](https://github.com/quemsah/awesome-claude-plugins) README（按 Stars 排序的前 100 名，上游每日更新）构建离线缓存至 `~/.byoh/catalog.json`。单次抓取 + 解析（无逐页爬取），每个条目带有真实的 `stars`。默认先下载 **维护者预构建的 bundle**（每周的 GitHub Release 资产 — 数秒），仅当其不可达时才直接解析 README。索引完成后，`catalog search` 和 `catalog vendor` 完全在本地离线运行。在 S2 向导采访过程中，`profile_interview` 会自动在响应中附带 `catalog_suggestions`（最多 5 个按 genre 匹配的插件推荐），LLM 无需额外调用工具即可向用户推荐相关插件。

  `catalog vendor` **在引入时自动丰富目录缓存**：克隆插件仓库后，从 `.claude-plugin/plugin.json` 提取 `license` 和 `keywords`，并记录已解析的 `genre`。仅当缓存值为 `"unknown"` 或为空时才写回 `catalog.json`，每次引入操作都会让 `catalog search` 结果更加丰富。LLM 代理可通过 `catalog_search` / `catalog_vendor` MCP 工具自主完成搜索 → 引入全流程，用户也可直接通过 CLI 手动指定。

  ```bash
  # 一次性索引 — 优先下载预构建包，不可达时直接解析 README
  byoh catalog index                       # 优先包，README 兜底
  byoh catalog index --no-bundle           # 直接解析 README
  byoh catalog index --no-bundle --limit 20   # 仅前 20 项

  # 本地镜像测试覆盖:
  #   BYOH_BUNDLE_URL=http://localhost:18099/catalog.json.gz byoh catalog index

  # 离线搜索 — 无需网络
  byoh catalog search "test driven development" --genre developer --limit 5

  # 将找到的插件引入 registry/vendored/
  # 自动从克隆仓库中提取 license、keywords 和 genre
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
