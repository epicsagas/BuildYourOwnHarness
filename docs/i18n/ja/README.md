> この文書は [README.md](../../../README.md) の日本語訳です。英語版が権威ある原本です。

<div align="center">

**[English](../../../README.md)** | [한국어](../ko/README.md) | **日本語** | [简体中文](../zh-Hans/README.md) | [Español](../es/README.md) | [Deutsch](../de/README.md) | [Français](../fr/README.md) | [Português](../pt/README.md) | [Русский](../ru/README.md) | [العربية](../ar/README.md)

# BuildYourOwnHarness (BYOH)

### あなただけのAIエージェント

*汎用テンプレートではなく、あなたの役割・専門・目標に合わせてコンパイルされるハーネス。*

<img src="../../../assets/features.png" width="100%" alt="Build Your Own Harness">

</div>

多くのAIツールは固定のツールセットを渡して「あとはよろしく」と言うだけです。BYOHはその逆を行きます。短いインタビューであなたの仕事を学び、ワークフローにぴったりなパーソナルエージェントハーネス（スキル・メモリ・パイプライン）を生成します。

## こんな方に

- **開発者** — 自分のスタック・テストスタイル・リリースサイクルを最初から把握しているエージェントが欲しい
- **研究者** — 文献調査・引用管理・合成がひとつにつながったパイプラインが必要
- **クリエイター** — 自分の文体とプロジェクト構造を理解したライティングパートナーが欲しい
- **ビジネスアナリスト** — 汎用チャットではなく、意思決定フレームワークとレポートパイプラインが必要

「AIが自分のコンテキストをわかってくれたら」と思ったことがあるなら、BYOHはまさにそのためのツールです。

## 60秒で始める

```
1. byoh profile init me        # プロジェクトをスキャン（読み取り専用）
2. byoh profile interview me   # 役割と目標についての短い会話
3. byoh compile me             # 個人用ハーネスを生成
4. byoh install me             # Claude / Codex / agy にデプロイ
```

これだけです。次のセッションからホストがあなたのハーネスを自動的に読み込みます — エージェント・スキル・メモリ・パイプラインがすべてあなた仕様になった状態で。

**必要なものがすでにわかっている場合は**、コミュニティカタログから探せます。

```bash
byoh catalog index                              # トップ100プラグインリストを取得（数秒）
byoh catalog search "code review"               # 関連プラグインを検索
byoh catalog vendor anthropics/claude-code-review   # ハーネスに追加
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

### AIホストへのpluginロード

BYOHはClaude Code・Codex・agy に対応したポリグロットpluginとして動作します — ひとつのリポジトリで3つのホストすべてに対応。

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

pluginの初回ロード時に `byoh` バイナリが自動インストールされるため、Rustは必要ありません。

> **注意:** 現在リポジトリは非公開です。上記のパスを使用してください。公開後は `epicsagas/plugins` マーケットプレイスにも登録予定です。

## はじめてのハーネス — ステップごとに

### ステップ1 — プロファイル作成

```bash
byoh profile init me --paths ./src ./docs   # プロジェクトを自動スキャン
byoh profile interview me                   # 約5分のインタビュー
byoh profile confirm me --genre developer   # ジャンルを確定
```

BYOHはあなたの役割・専門レベル・使用ツール・30日間の目標を質問します。インタビューは適応型 — 研究者には開発者とは異なる質問が出ます。

### ステップ2 — コンパイルとインストール

```bash
byoh compile me                    # HarnessBundle を生成（検証・ゲート済み）
byoh render me --target claude     # または: codex | agy | all
byoh install me                    # dist/ に安全にインストール
```

### ステップ3 — 実行と進化

```bash
byoh run me       # ハーネスを有効にして起動
byoh evolve me    # セッションフィードバックをもとにハーネスを改善
```

`evolve` は3段階ゲート（Critic / Seesaw / Stagnation）を通過します。このゲートは一切バイパスできないため、進化プロセスは安全で監査可能です。

## プラグインカタログ

カタログには、スター数順のトップ100 Claudeプラグイン（毎日自動更新）が収録されています。ターミナルを離れずにコミュニティスキルを発見・追加できます。

```bash
# 初回インデックス — 事前ビルド済みバンドルを数秒でダウンロード
byoh catalog index

# インデックス後はオフラインで検索可能
byoh catalog search "memory" --genre developer --limit 5

# ハーネスにpluginを追加
# ライセンス・キーワード・ジャンルはクローンしたリポジトリから自動検出
byoh catalog vendor obra/superpowers --genre developer
```

LLMエージェントは `catalog_search` / `catalog_vendor` MCPツールを使ってこのフロー全体を自律実行できます。CLIから手動で操作することも可能です。

## エージェント主導モード

`byoh serve` でstdio MCPサーバーを起動すると、あなたがコマンドを打つ代わりにAIホストがBYOHの14個のツールを直接呼び出します — 会話そのものがインタビュー・ウィザード・実行になります。

```bash
byoh serve   # Claude / Codex / agy が接続してすべてを操作
```

利用可能なツール: `profile_create`、`profile_scan`、`profile_interview`、`profile_confirm`、`compile`、`evolve_cycle`、`rag_index`、`rag_search`、`genre_list`、`registry_clone_skill`、`catalog_search`、`catalog_vendor` など。

## CLIリファレンス

```bash
# プロファイル
byoh profile init <slug> [--paths ...]      # プロジェクトスキャン（非破壊）
byoh profile interview <slug>               # ガイド付きインタビュー
byoh profile confirm <slug> --genre <g>     # プロファイルを確定

# ビルド
byoh compile <slug> [--dry-run]             # 検証 + HarnessBundle 生成
byoh render <slug> --target <host>          # claude | codex | agy | all
byoh install <slug> [--host <dir>]          # dist/ またはライブpluginディレクトリにデプロイ

# 実行・進化
byoh run <slug>
byoh evolve <slug>

# コミュニティスキル
byoh vendor add <src> --genre <g> --id <id> [--keywords k1,k2] [--trust] [--sha <s>]
byoh vendor list
byoh vendor remove <id> --genre <g>

# カタログ
byoh catalog index [--no-bundle] [--limit N]
byoh catalog search "<クエリ>" [--genre <g>] [--tags k1,k2] [--limit N]
byoh catalog vendor <owner/repo> [--genre <g>] [--keywords k1,k2]

# 知識ベース（RAG）
byoh index <slug> [--corpus <dir>] [--force]
byoh search <slug> "<クエリ>" [--genre <g>] [--k N]
```

## 内部の仕組み

BYOHの合成エンジンはプロファイルのタグとスキルレジストリを照合し、依存関係を解決した順序でパイプラインを構成、サポートされているホストのネイティブフォーマットに変換できる `HarnessBundle`（git-ready）を出力します。

- **4リングセキュリティモデル** — 組み込みスキル（Ring 1）からコミュニティ/未信頼スキル（Ring 4）まで、段階的なバリデーション
- **3段階ゲート進化** — `evolve` の各サイクルはCritic（品質）・Seesaw（回帰）・Stagnation（停滞）ゲートを通過。バイパス不可
- **永続RAG** — 変更時のみ増分再埋め込み（`+追加 ~変更 -削除`）。検索は保存済みインデックスを再利用し、再埋め込みなし
- **目標指向パイプライン** — 30日間の目標（プロダクトローンチ・研究レポート・セキュアシップなど）を宣言すると、対応するスキルラダーが自動的にオーバーレイ

アーキテクチャ: ヘキサゴナル — `domain / ports / adapters / application / compiler / evolve / templates / deploy / i18n / obs / security / cli`。詳細は `AGENTS.md` を参照してください。

## ビルド・開発

```bash
cargo build --release
cargo clippy --all-targets -- -D warnings
cargo test                        # ユニット + e2e
cvp                               # 並列実行: check → clippy → test → fmt → build
```

オプションフィーチャー: `--features mcp`（MCPサーバー）、`--features native-rag`（ローカル埋め込み）、`--features rag-openai`（OpenAI埋め込み）。リリースバイナリはすべてのフィーチャーを含みます。

## ライセンス

Apache-2.0.
