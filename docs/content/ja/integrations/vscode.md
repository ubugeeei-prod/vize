---
title: VSコード
---

<!-- Generated translation; source: integrations/vscode.md -->

# VS コードの統合

> **⚠️ 進行中の作業:**Vize のエディターのサポートはまだ実験段階です。

> **重要:**Vue エディターの日常的なサポートについては、引き続き公式の Vue 言語ツールを使用してください。
> (`vuejs/language-tools`) とりあえず。 Vize は増分オプトイン評価用に設計されています。

リポジトリには、2 つの実験的な VS Code 拡張機能が含まれています。

- **Vize**— `vize lsp` による Vue 言語サポート
- **Vize Art**— Musea `*.art.vue` ファイルの構文ハイライト

VS Code マーケットプレイスからインストールします。

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

`*.art.vue` に Vize のホバー、完了、定義への移動、および
構文の強調表示に加えて参照のサポート。

## Vize 拡張機能

Vize 拡張機能は `vize lsp` で開始され、特定の機能バンドルをオプトインできます。
拡張機能が無効になったまま、または機能が有効になっていない状態で Vue ファイルを開くと、拡張機能はワンクリックで推奨されるワークスペース設定を提供するようになりました。これにより、ホバー、ジャンプ、診断が黙ってオフのままになることがなくなります。
この設定により、現在のワークスペースに `vize.enable`、`vize.lint.enable`、`vize.typecheck.enable`、および `vize.editor.enable` が書き込まれます。
`vize.enable: true` のみを手動で設定した場合、Vize はその推奨診断も使用し、
空の言語サーバーを起動する代わりに、エディター プロファイルを使用します。
Vize ステータス バー項目により `Vize: Show Status` が開き、プロファイル スイッチャー、サーバーが表示されます。
バイナリ ピッカー、再起動アクション、設定、ログを 1 か所から管理できます。

### 推奨される開始点

```json
{
  "vize.enable": true,
  "vize.lint.enable": true,
  "vize.typecheck.enable": false,
  "vize.editor.enable": false,
  "vize.formatting.enable": false
}
```

これにより、最初に lint 診断が有効になり、ナビゲーション、補完、およびフォーマット設定はユーザーに委ねられます。
既存の Vue ツール。

### 共通設定

| 設定                         | 目的                                         |
| ---------------------------- | -------------------------------------------- |
| `vize.enable`                | 拡張機能と言語サーバーを有効にする           |
| `vize.serverPath`            | `vize` 実行可能パスをオーバーライドします。  |
| `vize.lint.enable`           | lint 診断を有効にする                        |
| `vize.typecheck.enable`      | タイプ認識診断とバックエンド機能を有効にする |
| `vize.editor.enable`         | エディター支援バンドルを有効にする           |
| `vize.completion.enable`     | 補完を有効にする                             |
| `vize.formatting.enable`     | ドキュメントの書式設定を有効にする           |
| `vize.definition.enable`     | 定義への移動を有効にする                     |
| `vize.references.enable`     | 参照を有効にする                             |
| `vize.hover.enable`          | ホバーを有効にする                           |
| `vize.codeActions.enable`    | lint クイックフィックスを有効にする          |
| `vize.semanticTokens.enable` | セマンティック トークンを有効にする          |
| `vize.trace.server`          | LSP 通信をトレースする                       |

### 便利なコマンド

| コマンド                                  | 目的                                                  |
| ----------------------------------------- | ----------------------------------------------------- |
| `Vize: Show Status`                       | ステータスとセットアップのアクション ハブを開きます。 |
| `Vize: Enable Recommended Profile`        | lint、型チェック、およびエディター支援を有効にする    |
| `Vize: Enable Lint-Only Profile`          | 他のツールを使用したまま診断を有効にする              |
| `Vize: Select Language Server Executable` | ファイルピッカーから `vize.serverPath` を設定します。 |
| `Vize: Disable Language Server`           | 現在の構成ターゲットの Vize を停止する                |
| `Vize: Restart Language Server`           | 言語サーバーを再起動します。                          |
| `Vize: Show Output Channel`               | 拡張機能と LSP ログを表示する                         |

### 拡張機能が使用するもの

```text
VS Code
  ↕ Language Server Protocol
vize lsp (vize_maestro)
  → vize_armature
  → vize_croquis
  → vize_patina
  → vize_canon
  → vize_glyph
```

### ソースまたは VSIX からのインストール

[Vite+ インストール ガイド](https://viteplus.dev/guide/install) から `vp` を 1 回インストールしてから、次の手順を実行します。

```bash
git clone https://github.com/ubugeeei-prod/vize.git
cd vize
cd editors/vscode
vp install -- --ignore-workspace
vp pack
vp exec vsce package --no-dependencies --out dist/vize.vsix
code --install-extension dist/vize.vsix
```

## Vize アート拡張機能

`Vize Art` は、Musea `*.art.vue` ファイルの構文強調表示を提供します。
マーケットプレイス拡張機能 ID は `vize.vize-art` です。

それは以下を認識します:

- `<art>` メタデータ ブロック
- `<variant>` ブロック
- 標準の Vue `<template>`、`<script>`、および `<style>` セクション

## 他の編集者

`vize lsp` は言語サーバー プロトコルに従っており、Neovim、Helix、
ゼッドとEmacs。

Neovim セットアップの例:

```lua
require("lspconfig").vize.setup({
  cmd = { "vize", "lsp" },
  filetypes = { "vue" },
  init_options = {
    lint = true,
    typecheck = true,
    editor = true,
  },
})
```

`editor = true` は、ホバー、完了、ジャンプ、参照、シンボルをテストする最も簡単な方法です
一緒に。 tsgo などの別の TypeScript サーバーがプロジェクト診断を所有している場合は、
`typecheck = false` を選択し、評価したい Vue 固有の機能のみをオンにします。
