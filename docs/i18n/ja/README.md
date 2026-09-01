> この文書は [README.md](../../../README.md) の日本語訳です。英語版が権威ある原本です。

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | **日本語** | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### あなたを中心とした AI エージェント

*汎用テンプレートではなく、あなたの役割・専門分野・目標に合わせてコンパイルされるハーネス。*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

ほとんどの AI セットアップは、固定のツールセットを渡して「あとは頑張って」と投げっぱなしにします。BYOH はその発想を裏返します。あなたにインタビューし、実際に何をしているのかを学んで、スキル・エージェント・目標パイプラインを備えた、最初からワークフローにフィットするパーソナライズ済みのエージェントハーネスを生成します。

## こんな方に

- **開発者** — 自分のスタック・テストスタイル・デリバリーのペースを最初から把握しているエージェントが欲しい
- **研究者** — 文献レビュー・引用追跡・統合が一つに繋がったものが必要
- **クリエイター** — 自分の文体やプロジェクト構造に合った執筆パートナーが欲しい
- **ビジネスアナリスト** — 生のチャットではなく、意思決定フレームワークとレポートパイプラインが必要

「自分の AI が自分のコンテキストをちゃんと把握してくれたらな」と思ったことがあるなら、それこそ BYOH の役目です。

## 60秒でわかる仕組み

BYOH は、あなたがコマンドを打つのではなく、AI エージェントに駆動させるために作られています。プラグインをインストールしたら、あとはただ話しかけるだけ。会話そのものが*インタビュー*であり、ウィザードであり、ビルドです。

```
1. Install the plugin      # Claude Code / Codex / agy — auto-installs the binary
2. "Build me a harness"    # エージェントがインタビューし、ビルドし、足りない部分は
                           # 自ら埋めてインストールまで — すべて会話の中で
```

次のセッションでは、ホストがハーネスを自動的に読み込みます — あなたに合わせて調整されたエージェント・スキル・目標パイプラインが揃います。

## プラグインをインストール（推奨）

**Claude Code、Codex、agy** をお使いですか？ プラグインをインストールしてください。MCP サーバーを同梱し、**初回ロード時にバイナリを自動インストール**します — Rust ツールチェーンも手動セットアップも不要です:

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@byoh
```

**agy (Antigravity):**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add epicsagas/BuildYourOwnHarness
codex plugin add byoh@byoh
```

### 他の MCP 対応ホストをお使いですか？

BYOH は MCP を話すので、Cursor、Zed、Continue などでも動作します。[バイナリ](#バイナリを直接インストール)を一度インストールし、ホストからサーバーを参照してください:

```bash
byoh serve   # stdio MCP server
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Note:** このリポジトリは `byoh` という名前の独立マーケットプレイス(.claude-plugin/marketplace.json)を同梱しており、ハブなしで単体インストールできます。

## バイナリを直接インストール

プラグインを（バイナリを自動インストールするもの）使わない場合や、非プラグインの MCP ホストで BYOH を使う場合にのみ必要です。

### macOS / Linux

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

### Windows (PowerShell)

```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

### ソースから

```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # verify
```

## エージェント駆動モード — メインの道

ホストを接続したら、コマンドを打つ必要はありません — ただ話しかけるだけです。エージェントが BYOH の MCP ツールを直接呼び出し、会話そのものが*インタビュー*であり、ビルドであり、進化（evolve）サイクルになります:

> **あなた:** *今月決済 API を出荷するバックエンドの Go 開発者です。ハーネスを構築してください。*
>
> **エージェント:** *（`profile_scan` でリポジトリをスキャンし、`profile_interview` で的を絞った質問をいくつか投げ、ジャンルを `developer` に固定）* → `build` が `HarnessBundle` を合成し、各スキルを `matched` / `authored` / `skeleton` に分類 → プロファイルが必要とする skeleton（例えば決済専用の検証スキル）があれば、エージェントがその場で `author_skill` で執筆してから確認のためもう一度 `build` → エージェント・スキル・secure-ship の目標パイプラインを Claude Code にインストール。執筆内容は再ビルドしても保持されます。完了 — 次のセッションでは、エージェントはすでにあなたのスタックを話しています。

同じフローを、推奨されるツールの呼び出し順で示すと:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → build → (author_skill / author_doc to fill skeletons) → build → install_plugin
```

`build` はバンドルを合成し（compile + preset 注入 + static gate）、各スキルを `matched`（実プリセット本文が注入済み）か `skeleton`（ジャンルテンプレートのプレースホルダー）に分類します — エージェントはこれを見て、今すぐインストールするかプロファイルを見直すかを判断します。

利用可能なツール: `profile_read`, `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `build`, `author_skill`, `author_doc`, `enable_hook`, `list_overrides`, `delete_override`, `render_plugin`, `install_plugin`, `catalog_search`, `catalog_vendor`.

エージェントに手取り足取り案内してほしいですか？ *"build my harness"* とだけ言えば、同梱の `byoh-guide` エージェントがフロー全体を取り仕切ります。

## プラグインカタログ

カタログは [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) の README から構築しています — コミュニティが管理する、上位 100 個の Claude プラグインリポジトリをスター順に並べたリストです。BYOH は事前ビルド済みバンドル（**毎週**月曜 03:17 UTC に再構築）を同梱しているため `byoh catalog index` は数秒で終わります。`--no-bundle` を渡せば上流のリストを直接パースします。

```bash
# One-time index — downloads a prebuilt bundle in seconds
byoh catalog index

# Search offline — no network needed after indexing
byoh catalog search "memory" --genre developer --limit 5

# Add a plugin to your harness
# license, keywords, and genre are auto-detected from the cloned repo
byoh catalog vendor obra/superpowers --genre developer
```

LLM エージェント（`catalog_search` / `catalog_vendor` MCP ツール経由）はこのフロー全体を自律的に実行できます — *"add a memory plugin to my harness"* — し、もちろん CLI から直接駆動することも可能です。

いくつかのコンパニオンツールは、**参照資料**（依存関係ではなく）として検索結果にシードされています。BYOH 自身の実行レイヤーのツール — [alcove](https://github.com/epicsagas/alcove)（ドキュメントサーバー）、[obsidian-forge](https://github.com/epicsagas/obsidian-forge)（ボールト自動化）、[epic-harness](https://github.com/epicsagas/epic-harness)（フック/スキルランタイム） — は文脈に応じて現れ（"doc server" / "search backend" のクエリで alcove が見つかります）、関連時にエージェントが推奨できるようになっています。本当に欲しい場合にのみ vendor してください。いずれにせよバンドルは依存関係なしで出荷されます。

## パワーユーザー: CLI（任意）

上記のフローはすべてターミナルからも実行できます。CLI は**補助的**です — スクリプト・CI・チャットせずに進めたい場合に便利ですが、エージェント駆動のパスが本来の想定です。

### 最初のハーネス — CLI から

```bash
byoh profile init me --paths ./src ./docs   # auto-scans your project
byoh profile confirm me --genre developer   # lock in your genre (+ optional --goal)

byoh render me --target claude              # 合成（compile + preset 注入 + static gate）して HarnessBundle を書き出す; or: codex | agy | all (default: all)
byoh install me --scope local               # render to dist/, activate into this project's .claude/ only
byoh install me --scope global              # ...or ~/.claude + ~/.codex + ~/.gemini (was --host)
byoh install me --scope publish             # ...or add LICENSE + .gitignore and print git instructions
```

インタビュー自体はエージェント駆動です（`profile_interview` MCP ツール）。会話がインタビューであるため、対話型の CLI インタビューは存在しません。build の static gate（Critic / Seesaw / Stagnation の安全ゲート、MCP スキーマ、hook 入力）は常に実行され、決してバイパスできません — したがってバンドルは出荷前に構造的に有効であることが保証されます。インストール後の改善はツール呼び出しではなく、後続セッションでの対話的な振り返りです。

## 内部の仕組み

BYOH の合成エンジンは、プロファイルのタグをスキルレジストリと照合し、依存関係を解決したパイプラインに並べ、サポートされている任意のホストのネイティブ形式にレンダリングされる git 対応のアーティファクト ── `HarnessBundle` を出力します。

- **4 リングセキュリティモデル** — ライフサイクル仕様（Ring 0）と組み込みのパイプラインスキル（Ring 1）から、コミュニティ/非信頼スキル（Ring 3）まで、それぞれ段階的に厳格になる検証。vendor されたスキルは sha256 でピン留めされ、読み取り + 組み込み時に検証される
- **3 ゲート安全基盤** — すべてのビルドの static gate は Critic（品質）・Seesaw（回帰）・Stagnation（停滞）の各ゲートの存在を確認する。バイパス不可
- **ゴール指向のパイプライン** — 30 日の目標（プロダクトローンチ・研究レポート・secure ship など）を宣言すると、それに合うスキルラダーを自動で重ね合わせる

アーキテクチャ: ヘキサゴナル — `domain / ports / adapters / application / compiler / evolve / templates / deploy / catalog / mcp / i18n / security / cli`。完全なガイドは `AGENTS.md` を参照してください。

## 完全な CLI リファレンス

CLI は意図的に小さく保っています: 機械向けのエントリポイント（`serve`、CI での `catalog index`、メンテナー向け `vendor`）に、コアビルドフローのスクリプト可能なミラーを加えた構成です。インタビューと進化は MCP のみ（エージェント駆動）です。

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

プロファイルとカタログのキャッシュはデフォルトで `~/.byoh` に置かれます（`BYOH_HOME` で上書き可能）。

## ビルド & 開発

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # unit + e2e
cvp                               # parallel: check → clippy → test → fmt → build
```

`mcp` フィーチャー（stdio MCP サーバー）はデフォルトでオンです。BYOH は組み込みのナレッジベースを同梱しません — 検索には、生成したハーネスを [alcove](https://github.com/epicsagas/alcove) のようなドキュメントサーバーに向けてください。

## 謝辞

BYOH はいくつかのコミュニティの取り組みの上に成り立っています:

- **プラグインカタログ** — [`quemsah/awesome-claude-plugins`](https://github.com/quemsah/awesome-claude-plugins) から取得。上位 100 個の Claude プラグインリポジトリのスター順コミュニティリストです。これがなければカタログは存在しませんでした。
- **コンパニオンツール** — [alcove](https://github.com/epicsagas/alcove)（ドキュメントサーバー / RAG）、[Episteme](https://github.com/epicsagas/Episteme)（ナレッジグラフ）、[obsidian-forge](https://github.com/epicsagas/obsidian-forge)（ボールト自動化）と相互運用するよう設計されています。
- **OSS スタック** — [clap](https://docs.rs/clap)、[serde](https://serde.rs)、[ureq](https://docs.rs/ureq)、そして Rust エコシステムの上に構築されています。

カタログの項目と vendor されたコミュニティスキルはそれぞれ独自のライセンスを維持します（vendor 時に自動検出）。BYOH 自体は Apache-2.0 です。

## ライセンス

Apache-2.0.
