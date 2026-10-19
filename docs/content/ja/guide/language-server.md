---
title: 言語サーバー
description: vize lsp で、VS Code、Zed、Neovim、Vim、Helix、Emacs に Vue の診断・補完・ナビゲーション・整形を追加します。
---

# 言語サーバー

`vize lsp` は Vue 用の [Language Server Protocol](https://microsoft.github.io/language-server-protocol/)
サーバーです。Vize の lint ルール、型チェッカー、フォーマッターをエディターで使えるようにし、
`.vue` と Musea の `.art.vue` に補完とナビゲーションを提供します。

> [!WARNING]
> エディター対応は実験的な機能です。まずは 1 つのワークスペースで試してください。現在 Vue - Official
> （`vuejs/language-tools`）を使っている場合は、同じ問題が二重に報告されないよう
> [lint のみのプロファイル](#プロファイル)から始めてください。

## 機能

| 分類           | 内容                                                                                     |
| -------------- | ---------------------------------------------------------------------------------------- |
| 診断           | 入力中の [lint ルール](../rules/index.md)と[型エラー](./type-checking.md)、クイックフィックス |
| 補完           | テンプレート内のコンポーネント、props、イベント、ディレクティブ、スクリプトのバインディング |
| ナビゲーション | ホバー、定義へ移動、参照の検索、ドキュメントとワークスペースのシンボル                   |
| 編集           | リネーム、ファイル名変更時の更新、シグネチャヘルプ、インレイヒント、自動挿入、折りたたみ |
| 整形           | `.vue` ファイルの [Vize フォーマッター](./formatter.md)                                  |
| エコシステム   | Vue Router のルート名とパラメーター、Vue I18n のキー、Nuxt、Void Vue の補助              |

## プロファイル

どのエディター連携も、同じサーバーを 3 つのプロファイルのいずれかで起動します。エディターで Vue の
言語機能をどのツールに任せるかで選んでください。

| プロファイル  | 有効になる機能                                  | 選ぶ場面                                                   |
| ------------- | ----------------------------------------------- | ---------------------------------------------------------- |
| `recommended` | lint、型チェック、エディター機能、エコシステム  | Vue の言語サーバーを Vize だけにする                       |
| `lint`        | lint の診断だけ                                 | 型、補完、ナビゲーションは Vue - Official に任せる         |
| `off`         | なし                                            | アンインストールせずに Vize を止める                       |

`hover` と `references` だけ、のように個別の機能を有効にすることもできます。設定場所は以下の
エディターごとの説明を参照してください。

## `vize` バイナリを用意する

VS Code 拡張機能はサーバーを自動で検出またはダウンロードします。その他のエディターは `PATH` 上の
`vize lsp` を実行します。CLI をグローバルにインストールするか、
[GitHub Releases](https://github.com/ubugeeei-prod/vize/releases) のバイナリをエディターに指定してください。

```bash
npm install -g vize
vize --version
```

サーバーは CLI と同じプロジェクト設定（`vite.config.*` と `tsconfig.json`）を読み込みます。
[設定](./configuration.md)を参照してください。

## VS Code

Marketplace から拡張機能をインストールします。

```bash
code --install-extension ubugeeei.vize
code --install-extension vize.vize-art
```

`.vue` ファイルを開いてセットアップの案内を受け入れるか、**Vize: Show Status** を実行してプロファイルを
選びます。Vite+ プロジェクトでは `vp run editor:setup` がワークスペースの設定を書き込みます。
[Vite+ のエディター設定](./vite-plus-editor.md)を参照してください。すべての設定とコマンドは
[VS Code](../integrations/vscode.md) にまとめています。

## Zed

Zed に Vize 拡張機能をインストールします。Vize リポジトリのクローンでは、`vize ide zed --install` が
`editors/zed` を Zed にコピーします。そのうえで、Zed の `settings.json` でサーバーを選びます。

```json
{
  "languages": {
    "Vue.js": { "language_servers": ["vize", "..."] },
    "Art Vue": { "language_servers": ["vize"] }
  },
  "lsp": {
    "vize": {
      "initialization_options": {
        "lint": true,
        "typecheck": true,
        "editor": true,
        "ecosystem": true
      }
    }
  }
}
```

## Neovim

Neovim 0.11 以降が必要です。プラグインマネージャーで Vize リポジトリの `editors/nvim` を追加し、
次のように設定します。

```lua
require("vize").setup({ profile = "recommended" })
```

プラグインは `vim.lsp.config("vize", ...)` でサーバーを登録します。個別の機能を選ぶ場合は
`init_options` を渡します。

```lua
require("vize").setup({
  init_options = { lint = true, hover = true, references = true },
})
```

## Vim

Vim では [vim-lsp](https://github.com/prabirshrestha/vim-lsp) が必要です。

```vim
Plug 'prabirshrestha/vim-lsp'
Plug 'ubugeeei-prod/vize', { 'rtp': 'editors/vim' }

call vize#setup({'profile': 'recommended'})
```

## Helix

次の内容を `~/.config/helix/languages.toml` にマージします。`art-vue` 言語を含む完全なファイルは、
リポジトリの `editors/helix/languages.toml` にあります。

```toml
[language-server.vize]
command = "vize"
args = ["lsp"]

[language-server.vize.config]
editor = true
ecosystem = true
lint = true
typecheck = true
```

## Emacs

Vize は Eglot で動作します。

```elisp
(add-to-list 'load-path "/path/to/vize/editors/emacs")
(require 'vize)
(vize-setup-eglot 'recommended)
```

## その他のエディター

LSP クライアントであれば、stdio でサーバーを起動できます。プロセスを起動できないクライアントでは TCP を
使ってください。

```bash
vize lsp
vize lsp --port 9527
vize lsp --debug
```

プロファイルのスイッチ（`lint`、`typecheck`、`editor`、`ecosystem`、または `hover` などの個別機能）は
`initializationOptions` で渡します。

## トラブルシューティング

| 症状                                       | 確認すること                                                                  |
| ------------------------------------------ | ----------------------------------------------------------------------------- |
| 診断がまったく表示されない                 | プロファイルが `off` か、機能が 1 つも有効でない。VS Code では **Vize: Show Status** を実行する |
| 同じ型エラーが 2 回表示される              | Vue - Official も型をチェックしている。`lint` プロファイルにするか、どちらかを無効にする |
| `$t` などのグローバルが存在しないと報告される | 宣言ファイルが `tsconfig.json` の `include` の外にある。[型チェック](./type-checking.md#グローバルな型と自動インポート)を参照 |
| サーバーが起動しない                       | エディターの環境で `vize --version` が失敗している。バイナリのフルパスを設定する |

`--debug` を付けてサーバーを起動するか、VS Code の **Vize: Show Output Channel** でログを確認し、
[問題を報告する](./testing.md)ときに添えてください。
