> この文書は [README.md](../../../README.md) の日本語訳です。英語版が権威ある原本です。

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | **日本語** | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### あなだを中心とした AI エージェント

*汎用テンプレートではなく、あなたの役割・専門分野・目標に合わせてコンパイルされるハーネス。*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

ほとんどの AI セットアップは、固定のツールセットを渡して「あとはよしなに」と丸投げします。BYOH はその逆です。あなたにインタビューし、実際に何をしているのかを学んで、あなたのワークフローに最初からフィットするパーソナライズされたエージェントハーネス（スキル・メモリ・パイプライン）を生成します。

## こんな方に

- **開発者** — 自分のスタック・テストスタイル・リリースのペースを最初から把握しているエージェントが欲しい
- **研究者** — 文献レビュー・引用追跡・統合が一つに繋がったパイプラインが必要
- **クリエイター** — 自分の文体やプロジェクト構造に合った書き相手が欲しい
- **ビジネスアナリスト** — 生のチャットではなく、意思決定フレームワークとレポートパイプラインが必要

「AI が自分の文脈をちゃんとわかってくれたらな」と思ったことがあるなら、それが BYOH の仕事です。

## 60秒でわかる仕組み

BYOH は、あなたがコマンドを打つのではなく、AI エージェントに駆動させるために作られています。プラグインをインストールしたら、あとはただ話しかけるだけ。会話そのものがインタビューであり、ウィザードであり、ビルドです。

```
1. Install the plugin      # Claude Code / Codex / agy — バイナリを自動インストール
2. "Build me a harness"    # エージェントがリポジトリをスキャンして結果をコンパイル
```

次のセッションから、ホストがハーネスを自動的に読み込みます — エージェント・スキル・メモリ・パイプラインがすべてあなただけに調整された状態で。

## プラグインをインストール（推奨）

**Claude Code、Codex、agy** をお使いですか？ プラグインをインストールしてください。MCP サーバーを同梱し、**初回ロード時にバイナリを自動インストール**します — Rust ツールチェーンも手動セットアップも不要です:

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

### 他の MCP 対応ホストをお使いですか？

BYOH は MCP を話すので、Cursor、Zed、Continue などでも動作します。[バイナリ](#installation)を一度インストールして、ホストからサーバーに接続してください:

```bash
byoh serve   # stdio MCP サーバー
```

```json
{ "mcpServers": { "byoh": { "command": "byoh", "args": ["serve"] } } }
```

> **Note:** 現在リポジトリは非公開です。上記のパスを使用してください。公開後は共有の `epicsagas/plugins` マーケットプレースにも表示されます。

## エージェント駆動モード — メインの道

ホストを接続したら、コマンドを打つ必要はありません — ただ話しかけるだけです。エージェントが BYOH の MCP ツールを直接呼び出し、会話そのものがインタビュー・ビルド・進化サイクルになります:

> **あなた:** *今月リリースする決済 API のバックエンド Go 開発者です。ハーネスを作って。*
>
> **エージェント:** *(`profile_scan` でリポジトリをスキャン、`profile_interview` でいくつかの的を絞った質問、ジャンルを `developer` に固定)* → `HarnessBundle` をコンパイル → エージェント・スキル・メモリ・セキュアリリースパイプラインを Claude Code にインストール。完了 — 次のセッションでは、エージェントはもうあなたのスタックを理解しています。

同じフローを、推奨されるツールの呼び出し順で示すと:

```
profile_create → profile_scan → profile_interview → profile_confirm
           → rag_index / rag_search → compile → compile_dry_run
           → (任意) registry_clone_skill → (後で) evolve_cycle
```

利用可能なツール: `profile_create`, `profile_scan`, `profile_interview`, `profile_confirm`, `compile`, `compile_dry_run`, `evolve_cycle`, `genre_list`, `rag_index`, `rag_search`, `registry_clone_skill`, `catalog_search`, `catalog_vendor` など。

エージェントに案内してほしいですか？ *"build my harness"* とだけ言えば、同梱の `byoh-guide` エージェントがフロー全体を orchestrate します。

## プラグインカタログ

カタログは、上位 100 個の Claude プラグイン（スター順、毎日更新）をキュレーションして提供します。会話を抜けることなく、コミュニティスキルを発見して追加できます。

```bash
# 初回1回のみのインデックス — 事前ビルド済みバンドルを数秒でダウンロード
byoh catalog index

# インデックス後はオフラインで検索 — ネットワーク不要
byoh catalog search "memory" --genre developer --limit 5

# ハーネスにプラグインを追加
# license・keywords・genre はクローンしたリポジトリから自動検出
byoh catalog vendor obra/superpowers --genre developer
```

LLM エージェント（`catalog_search` / `catalog_vendor` MCP ツール経由）がこのフロー全体を自律的に実行できます — *"add a memory plugin to my harness"* — し、もちろん CLI から直接駆動することも可能です。

## パワーユーザー: CLI（任意）

上記のフローはすべてターミナルからも実行できます。CLI は**補助的**です — スクリプト・CI・チャットしたくない場合に便利ですが、エージェント駆動の道が本来の想定です。

### 最初のハーネス — CLI から

```bash
byoh profile init me --paths ./src ./docs   # プロジェクトを自動スキャン
byoh profile interview me                   # 約5分の会話
byoh profile confirm me --genre developer   # ジャンルを確定

byoh compile me                             # HarnessBundle を生成（検証 + ゲート通過）
byoh render me --target claude              # or: codex | agy | all
byoh install me                             # dist/ に安全にインストール

byoh run me                                 # ハーネスを有効化した状態で起動
byoh evolve me                              # セッションフィードバックに基づいてハーネスを改善
```

BYOH は役割・専門レベル・ツール・30日目標について質問します。インタビューは適応します — 研究者は開発者とは異なる質問を受けます。`evolve` は迂回できない 3 ゲートサイクル（Critic / Seesaw / Stagnation）を実行するので、進化は安全で監査可能です。

## 内部の仕組み

BYOH の合成エンジンは、プロファイルのタグをスキルレジストリと照合し、依存関係を解決した順序のパイプラインに並べ、対応するすべてのホストのネイティブ形式にレンダリングされる `HarnessBundle` を生成します。

- **4 リングセキュリティモデル** — 組み込みスキル（Ring 1）からコミュニティ/非信頼スキル（Ring 4）まで、段階的に検証の厳しさが上がる
- **3 ゲート進化** — 毎回の `evolve` サイクルは Critic（品質）・Seesaw（回帰）・Stagnation（停滞）のゲートをすべて通過。迂回不可
- **ゴール指向のパイプライン** — 30日目標（製品リリース・研究レポート・セキュアリリース…）を宣言すると、それに合うスキルラダーを自動で重ね合わせる

アーキテクチャ: ヘキサゴナル — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`. 完全なガイドは `AGENTS.md` を参照してください。

## 完全な CLI リファレンス

```bash
# プロファイル
byoh profile init <slug> [--paths ...]      # 非破壊的なプロジェクトスキャン
byoh profile interview <slug>               # ガイド付きインタビュー
byoh profile confirm <slug> --genre <g>     # プロファイルを確認・固定

# ビルド
byoh compile <slug> [--dry-run]             # 検証 + HarnessBundle 生成
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # dist/ または稼働中のプラグインディレクトリにデプロイ

# 実行 & 進化
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

## Installation

プラグインを（バイナリを自動インストールするもの）使わない場合や、非プラグインの MCP ホストで BYOH を使う場合にのみ必要です。

### バイナリ（Rust ツールチェーン不要）

**macOS / Linux:**
```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/epicsagas/BuildYourOwnHarness/releases/latest/download/install.sh | sh
```

**Windows (PowerShell):**
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

## ビルド & 開発

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # ユニット + e2e
cvp                               # 並列: check → clippy → test → fmt → build
```

`mcp` フィーチャー（stdio MCP サーバー）はデフォルトでオンです。BYOH は組み込みのナレッジベースを同梱しません — 検索には、生成したハーネスを [alcove](https://github.com/epicsagas/alcove) のようなドキュメントサーバーに向けてください。

## ライセンス

Apache-2.0.
