> この文書は [README.md](../../README.md) の日本語訳です。英語版が権威ある原本であり、より新しい可能性があります。
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
byoh evolve <slug>                       # 3ゲート進化サイクル
byoh catalog index [--limit N]           # quemsah トップ100 README をパース → ~/.byoh/catalog.json
byoh catalog search "<クエリ>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

### エージェント主導モード（MCPサーバー）

`byoh serve`（`--features mcp`）はstdio MCPサーバーを起動し、LLMエージェントが**BYOHを駆動**します — CLIは副次的な手段になります（制御の逆転）。14個のツール（`profile_*`、`rag_*`、`genre_list`、`compile`、`evolve_cycle`、`registry_clone_skill`、`catalog_search`、`catalog_vendor`）が`tools/list`で検索可能です。会話そのものがインタビュー/ウィザードです。

```bash
cargo build --release --features mcp
byoh serve
```

## コア：シンセシス・ベンダリング・カタログ

- **シンセシスエンジン** — `synthesize(profile)` はプロファイルタグに対してレジストリスキルをマッチングし、パイプラインに整序して3ゲートの再パスを強制します（バイパス不可）。30日間の目標がマッチした場合、ゴール指向パイプライン（product-launch / decision / research-report / secure-ship / …）がスキルラダーとエージェントセットを重ねます。
- **コミュニティスキルベンダリング**（RFC M3） — `byoh vendor add` は外部の `SKILL.md`（ローカルパスまたはgit URL）を取得し、静的検証とsha256を実行して、`build.rs` によりビルド時に**Ring 3**（最も制限された）に組み込みます。外部スキルは非信頼コードとしてシンセシスに参加します。
- **プラグインカタログ** — `byoh catalog index` はキュレーションされた [quemsah/awesome-claude-plugins](https://github.com/quemsah/awesome-claude-plugins) README（Stars順トップ100、上流で毎日更新）から `~/.byoh/catalog.json` にオフラインキャッシュを構築します。1回の取得＋パース（ページ別クロール不要）で、各エントリに実際の `stars` が含まれます。デフォルトでは **メンテナー作成のバンドル**（週次の GitHub Release アセット — 数秒）を先にダウンロードし、取得できない場合のみ README を直接パースします。インデックス後、`catalog search` と `catalog vendor` は完全オフラインで動作します。S2ウィザードインタビュー中、`profile_interview` は自動的に `catalog_suggestions`（ジャンルにマッチした最大5件のプラグイン推薦）を含めます — 追加のツール呼び出し不要です。

  `catalog vendor` は**ベンダー時にカタログキャッシュを自動的に拡充**します: プラグインレポをクローン後、`.claude-plugin/plugin.json` から `license` と `keywords` を抽出し、確定した `genre` を記録します。これらはキャッシュ値が `"unknown"` または空の場合にのみ `catalog.json` に書き戻されます。LLMエージェントは `catalog_search` / `catalog_vendor` MCP ツールで検索 → ベンダーの流れを完全自律実行でき、ユーザーがCLIで直接指定することもできます。

  ```bash
  # 1回限りのインデックス作成 — バンドル優先、不可時はREADME直接パース
  byoh catalog index                       # バンドル優先、READMEフォールバック
  byoh catalog index --no-bundle           # READMEを直接パース
  byoh catalog index --no-bundle --limit 20   # 上位20件のみ

  # ローカルミラーテスト用オーバーライド:
  #   BYOH_BUNDLE_URL=http://localhost:18099/catalog.json.gz byoh catalog index

  # オフライン検索 — ネットワーク不要
  byoh catalog search "test driven development" --genre developer --limit 5

  # 見つかったプラグインを registry/vendored/ にベンダリング
  # license、keywords、genreをクローンしたレポから自動抽出
  byoh catalog vendor obra/superpowers --genre developer
  ```

## Status

Rust implementation of the generation layer: profiler + interview + genre templates + compiler (4-ring, MCP-tool codegen, static gate) + evolution engine + self-contained RAG (optional `native-rag` feature) + MCP server (optional `mcp` feature). See `AGENTS.md` for the architecture guide.

The RAG layer is a **persistent knowledge base**: `byoh index` saves the genre index + a corpus sidecar under `$BYOH_HOME/indexes/`, and a later `byoh search` (or the `rag_search` MCP tool) with no `--corpus` reuses it via `load_index` — no re-embedding. Re-indexing is **incremental** — a content-hash manifest re-embeds only added/changed docs and drops removed ones (reported as `+a ~c -r`); `--force` does a full rebuild.

## License

Apache-2.0.
