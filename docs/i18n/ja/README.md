> この文書は [README.md](../../../README.md) の日本語訳です。英語版が権威ある原本です。

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | **日本語** | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### あなただけのAIエージェント

*汎用テンプレートではなく、あなたの役割・専門・目標に合わせてコンパイルされるハーネス。*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

多くのAIツールは固定のツールセットを渡して「あとはよろしく」と言うだけです。BYOHはその逆を行きます。あなたにインタビューし、実際の仕事を学んで、ワークフローにぴったりなパーソナルエージェントハーネス（スキル・メモリ・パイプライン）を生成します。

## こんな方に

- **開発者** — 自分のスタック・テストスタイル・リリースサイクルを最初から把握しているエージェントが欲しい
- **研究者** — 文献調査・引用管理・合成がひとつにつながったパイプラインが必要
- **クリエイター** — 自分の文体とプロジェクト構造を理解したライティングパートナーが欲しい
- **ビジネスアナリスト** — 汎用チャットではなく、意思決定フレームワークとレポートパイプラインが必要

「AIが自分のコンテキストをわかってくれたら」と思ったことがあるなら、BYOHはまさにそのためのツールです。

## 60秒で始める

BYOHはAIエージェントが駆動するように作られています。インストールして、MCPでホストを接続し、あとはただ話しかけるだけ — 会話そのものがインタビュー・ウィザード・ビルドになります。

```
1. Install byoh              # ワンラインインストール（後述）
2. Connect your host via MCP # byoh serve — 任意のMCP対応エージェント
3. "Build me a harness"      # エージェントがリポジトリをスキャンして結果をコンパイル
```

次のセッションからホストがハーネスを自動的に読み込みます — エージェント・スキル・メモリ・パイプラインがすべてあなた仕様になった状態で。

**ターミナル派ですか?** 同じフローをCLIから実行できます。
```
byoh profile init me        # プロジェクトをスキャン（非破壊・読み取り専用）
byoh profile interview me   # 役割と目標についての短い会話
byoh compile me             # 個人用ハーネスを生成
byoh install me             # Claude / Codex / agy にデプロイ
```

**必要なものがすでにわかっている場合は**、コミュニティカタログから探せます。
```bash
byoh catalog index                                 # トップ100プラグインリストを取得（数秒）
byoh catalog search "code review"                  # 関連プラグインを検索
byoh catalog vendor anthropics/claude-code-review  # ハーネスに追加
```

## インストール

### バイナリ（推奨 — Rustツールチェイン不要）

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows（PowerShell）:**
```powershell
irm https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.ps1 | iex
```

**ソースから:**
```bash
cargo install byoh --git https://github.com/epicsagas/BuildYourOwnHarness
```

```bash
byoh --version   # 確認
```

### AIホストを接続

BYOHはMCPに対応しているため、MCP対応エージェントならどれでも駆動できます。上記のバイナリをインストールし、サーバーを起動すれば、ホストはBYOHの全ツールを直接呼び出せます。

```bash
byoh serve   # stdio MCPサーバー
```

**その他のエージェント**（Cursor・Zed・Continue など）の場合は、ホストのMCP設定に `byoh` を追加してください。
```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

**Claude Code・Codex・agy** をお使いなら、代わりにプラグインをインストールしてください — MCPサーバーを同梱し、初回ロード時にバイナリを自動インストールします（Rust不要）。

**Claude Code:**
```bash
claude plugin marketplace add epicsagas/BuildYourOwnHarness
claude plugin install byoh@epicsagas
```

**agy（Antigravity）:**
```bash
agy plugin install /path/to/BuildYourOwnHarness
agy plugin enable byoh
```

**Codex:**
```bash
codex plugin marketplace add /path/to/BuildYourOwnHarness
codex plugin add byoh@epicsagas
```

> **注意:** 現在リポジトリは非公開です。上記のパスを使用してください。公開後は共有の `epicsagas/plugins` マーケットプレイスにも登録予定です。

## エージェント主導モード

ホストを接続すれば、コマンドを打つ必要はありません — ただ話すだけです。エージェントがBYOHの14個のツールを直接呼び出し、会話そのものがインタビュー・ビルド・進化サイクルになります。

利用可能なツール: `profile_create`、`profile_scan`、`profile_interview`、`profile_confirm`、`compile`、`evolve_cycle`、`genre_list`、`registry_clone_skill`、`catalog_search`、`catalog_vendor` など。

## はじめてのハーネス — CLIから

同じステップをターミナルから実行できます。

### ステップ1 — プロファイル
```bash
byoh profile init me --paths ./src ./docs   # プロジェクトを自動スキャン
byoh profile interview me                   # 約5分のインタビュー
byoh profile confirm me --genre developer   # ジャンルを確定
```

BYOHはあなたの役割・専門レベル・使用ツール・30日間の目標を質問します。インタビューは適応型 — 研究者には開発者とは異なる質問が出ます。

### ステップ2 — コンパイルとインストール
```bash
byoh compile me          # HarnessBundle を生成（検証・ゲート済み）
byoh render me --target claude   # または: codex | agy | all
byoh install me          # dist/ に安全にインストール
```

### ステップ3 — 実行と進化
```bash
byoh run me              # ハーネスを有効にして起動
byoh evolve me           # セッションフィードバックをもとにハーネスを改善
```

`evolve` は3段階ゲート（Critic / Seesaw / Stagnation）を通過します。このゲートは一切バイパスできないため、進化プロセスは安全で監査可能です。

## プラグインカタログ

カタログには、スター数順のトップ100 Claudeプラグイン（毎日自動更新）が収録されています。ターミナルを離れずにコミュニティスキルを発見・追加できます。

```bash
# 初回インデックス — 事前ビルド済みバンドルを数秒でダウンロード
byoh catalog index

# インデックス後はオフラインで検索可能（ネットワーク不要）
byoh catalog search "memory" --genre developer --limit 5

# ハーネスにプラグインを追加
# ライセンス・キーワード・ジャンルはクローンしたリポジトリから自動検出
byoh catalog vendor obra/superpowers --genre developer
```

LLMエージェント（`catalog_search` / `catalog_vendor` MCPツール経由）はこのフロー全体を自律実行できます — CLIから手動で操作することも可能です。

## CLIリファレンス

```bash
# プロファイル
byoh profile init <slug> [--paths ...]      # 非破壊プロジェクトスキャン
byoh profile interview <slug>               # ガイド付きインタビュー
byoh profile confirm <slug> --genre <g>     # プロファイルを確定

# ビルド
byoh compile <slug> [--dry-run]             # 検証 + HarnessBundle 生成
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # dist/ またはライブプラグインディレクトリにデプロイ

# 実行・進化
byoh run <slug>
byoh evolve <slug>

# コミュニティスキル
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# カタログ
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<query>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]
```

## 内部の仕組み

BYOHの合成エンジンはプロファイルのタグとスキルレジストリを照合し、依存関係を解決した順序でパイプラインを構成、サポートされているホストのネイティブフォーマットに変換できる `HarnessBundle`（git-ready）を出力します。

- **4リングセキュリティモデル** — 組み込みスキル（Ring 1）からコミュニティ/未信頼スキル（Ring 4）まで、段階的なバリデーション
- **3段階ゲート進化** — `evolve` の各サイクルはCritic（品質）・Seesaw（回帰）・Stagnation（停滞）ゲートを通過。バイパス不可
- **目標指向パイプライン** — 30日間の目標（プロダクトローンチ・研究レポート・セキュアシップなど）を宣言すると、対応するスキルラダーが自動的にオーバーレイ

アーキテクチャ: ヘキサゴナル — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`。詳細は `AGENTS.md` を参照してください。

## ビルド・開発

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # ユニット + e2e
cvp                               # 並列実行: check → clippy → test → fmt → build
```

`mcp` フィーチャー（stdio MCPサーバー）はデフォルトで有効です。BYOHは組み込みのナレッジベースを同梱しません — 検索には、生成したハーネスを [alcove](https://github.com/epicsagas/alcove) のようなドキュメントサーバーに向けてください。

## ライセンス

Apache-2.0.
