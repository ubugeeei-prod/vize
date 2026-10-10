---
title: VS Code
---

<!-- Reviewed translation; source: integrations/vscode.md -->

<span id="vs-コードの統合"></span>

# VS Code 統合

> **⚠️ 開発中:** Vize のエディターサポートはまだ実験段階です。

> **重要:** 通常の Vue 開発には、引き続き公式の Vue Language Tools（`vuejs/language-tools`）を使ってください。
> Vize は、機能を少しずつ有効にして評価できるように設計されています。

リポジトリには、2 つの実験的な VS Code 拡張機能が含まれています。

- **Vize**— `vize lsp` による Vue 言語サポート
- **Vize Art**— Musea `*.art.vue` ファイルの構文ハイライト

VS Code マーケットプレイスからインストールします。

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

`*.art.vue` でも Vize のホバー、補完、定義への移動、参照検索を使えます。
Vize Art は art ファイルの構文ハイライトを提供します。

## Vize 拡張機能

Vize 拡張機能は `vize lsp` を起動します。lint、型チェック、エディター支援などを個別に有効にできます。
拡張機能や各機能が無効のまま Vue ファイルを開くと、ワークスペースに推奨設定を適用する操作が表示されます。
この操作は `vize.enable`、`vize.lint.enable`、`vize.typecheck.enable`、`vize.editor.enable` を有効にします。
手動で `vize.enable: true` だけを設定した場合も、推奨される診断とエディター支援が有効になります。
ステータスバーの Vize をクリックすると `Vize: Show Status` が開きます。
プロファイルの変更、サーバー実行ファイルの選択、再起動、設定、ログの確認をまとめて行えます。

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

この設定では lint 診断だけを有効にします。定義への移動、補完、フォーマットは既存の Vue ツールに任せます。

次回リリースでは、専用の `vize.config.*` がない場合、ネイティブ LSP のフォーマットと Vue JSX 型チェックを既定で有効にする予定です。この lint 専用プロファイルでは、明示的な設定により型診断とフォーマットを無効にします。エディターの明示的な設定が優先されます。提供予定と専用設定との互換性は[プロジェクトの既定値](../guide/configuration.md#プロジェクトとエディターの既定値)を参照してください。

### 共通設定

| 設定                         | 目的                                         |
| ---------------------------- | -------------------------------------------- |
| `vize.enable`                | 拡張機能と言語サーバーを有効にする           |
| `vize.serverPath`            | 使用する `vize` 実行ファイルのパスを指定する |
| `vize.lint.enable`           | lint 診断を有効にする                        |
| `vize.typecheck.enable`      | 型情報を使う診断とバックエンド機能を有効にする |
| `vize.editor.enable`         | エディター支援バンドルを有効にする           |
| `vize.completion.enable`     | 補完を有効にする                             |
| `vize.formatting.enable`     | ファイルのフォーマットを有効にする           |
| `vize.definition.enable`     | 定義への移動を有効にする                     |
| `vize.references.enable`     | 参照検索を有効にする                         |
| `vize.hover.enable`          | ホバーを有効にする                           |
| `vize.codeActions.enable`    | lint クイックフィックスを有効にする          |
| `vize.semanticTokens.enable` | セマンティック トークンを有効にする          |
| `vize.trace.server`          | LSP 通信をトレースする                       |

### 便利なコマンド

| コマンド                                  | 目的                                                  |
| ----------------------------------------- | ----------------------------------------------------- |
| `Vize: Show Status`                       | 状態と初期設定の操作画面を開く |
| `Vize: Enable Recommended Profile`        | lint、型チェック、およびエディター支援を有効にする    |
| `Vize: Enable Lint-Only Profile`          | 他のツールを使用したまま診断を有効にする              |
| `Vize: Select Language Server Executable` | ファイル選択画面で `vize.serverPath` を指定する |
| `Vize: Disable Language Server`           | 現在の設定対象の Vize を停止する                |
| `Vize: Restart Language Server`           | 言語サーバーを再起動する |
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

<span id="vize-アート拡張機能"></span>

## Vize Art 拡張機能

`Vize Art` は、Musea の `*.art.vue` ファイルに構文ハイライトを提供します。
マーケットプレイス拡張機能 ID は `vize.vize-art` です。

次のブロックを認識します。

- `<art>` メタデータ ブロック
- `<variant>` ブロック
- 標準の Vue `<template>`、`<script>`、および `<style>` セクション

<span id="他の編集者"></span>

## 他のエディター

`vize lsp` は Language Server Protocol に対応しているため、Neovim、Helix、Zed、Emacs などでも利用できます。

Neovim の設定例です。

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

`editor = true` を指定すると、ホバー、補完、定義への移動、参照検索、シンボル検索をまとめて評価できます。
tsgo など別の TypeScript サーバーでプロジェクトの診断を実行する場合は、`typecheck = false` にして、
試したい Vue 固有の機能だけを有効にしてください。
