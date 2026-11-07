---
title: 既存ツールとの統合
description: Parcel、ESLint、Prettier、Babel、tsserver、任意の LSP エディターから Vize を使い、ほかのツールはそのまま残します。
---

# 既存ツールとの統合

Vize は、すでに使っているツールの中から利用できます。多くのプロジェクトでは、まず一つだけ選べば十分です。
このページのパッケージをすべて導入する必要はありません。同じ lint 診断・フォーマット・エディター機能が
二重に動かないよう、用途ごとに担当するツールを一つにします。

## 最初に一つ選ぶ

| 残したいワークフロー                   | 追加する Vize の機能                                                   | 公開状況                     |
| -------------------------------------- | ---------------------------------------------------------------------- | ---------------------------- |
| Parcel 2 の build                      | [`@vizejs/parcel-transformer`](#parcel)                                | 実験的・ワークスペース内のみ |
| `eslint`                               | [`eslint-plugin-vize`](#eslint-の-flat-config)                         | 実験的・ワークスペース内のみ |
| `prettier`                             | [`prettier-plugin-vize`](#prettier)                                    | 実験的・ワークスペース内のみ |
| Babel による JSX / TSX 変換            | [`@vizejs/babel-plugin`](#babel)                                       | npm で互換性プレビュー       |
| TypeScript 5.7–6.x の Language Service | [`@vizejs/typescript-vue-plugin`](#typescript-language-service-plugin) | 実験的・ワークスペース内のみ |
| LSP に対応した任意のエディター         | [`vize lsp`](#vs-code-以外のエディター)                                | `vize` CLI で利用可能        |

ワークスペース内のみのパッケージは、このリポジトリで実装・検証されていますが、npm レジストリからは
まだインストールできません。以下の例は、リポジトリの checkout で評価するときと、初回公開後に使う
設定形式を示します。プレビュー機能を導入する前に[パッケージのサポート区分](../stability.md#パッケージのサポート区分)を確認してください。

## Parcel

Parcel 2 がアプリケーションの build を担当する場合は、`@vizejs/parcel-transformer` を使います。この統合は
Vue SFC 全体をコンパイルします。Babel の JSX / TSX plugin とは別の機能であり、Vize の型チェッカーは
起動しません。現在は Parcel `^2.16.4` に対応しています。パッケージにはまだサポート対象の npm release が
ありませんが、初回公開後は次のようにインストールします。

```bash
npm install --save-dev parcel @vizejs/parcel-transformer
npm install vue
```

`.parcelrc` に SFC transformer、名前付き component pipeline、CSS continuation を登録します。

```json
{
  "extends": "@parcel/config-default",
  "reporters": ["./vize-parcel-dev-cache.cjs", "..."],
  "transformers": {
    "*.vue": ["@vizejs/parcel-transformer"],
    "vize-component:*.vue": ["@vizejs/parcel-transformer"],
    "*.css": ["@vizejs/parcel-transformer/style", "..."]
  }
}
```

`.parcelrc` と同じ場所に `vize-parcel-dev-cache.cjs` を作成します。

```js
module.exports = require("@vizejs/parcel-transformer/dev-cache");
```

Parcel は package の export を解決する前に reporter の package 名を検証するため、この相対 CommonJS adapter
から公開 subpath の reporter を読み込みます。`"..."` より前に配置してください。開発中にコンパイルエラーを
素早く修正したとき、古い bundle timestamp が再利用されることを防ぎます。

compiler の設定は `.vizerc.parcel.json`、または `package.json` の
`"@vizejs/parcel-transformer"` key に記述します。

```json
{
  "compiler": {
    "customElements": ["my-element"],
    "templateSyntax": "standard"
  },
  "transformAssetUrls": true
}
```

inline JavaScript / TypeScript、template の asset URL、scoped CSS、外部 style block、ホスト側の preprocessor、
source map、ブラウザー build、ネイティブ SSR build に対応しています。Sass、Less、Stylus を使う場合は、
対応する Parcel transformer と preprocessor を別途インストールしてください。Vize はそれらを自動では
ダウンロードしません。ファイル名、production mode、target、source-map 設定、scope identity は Parcel が
指定するため、Vize の option ではありません。

Vapor output は `"compiler": { "vapor": true }` で opt-in し、対応する Vue runtime と組み合わせます。
SFC CSS modules、外部 script / template block、custom block、生成される macro artifact は現在、明示的な
エラーになります。独立した `.module.css` import には Parcel 標準の CSS modules を使ってください。
開発時の HMR は変更された component を保守的に reload するため、inline SFC の編集で local state が
reset されることがあります。

## ESLint の flat config

flat config 形式の `eslint.config.mjs` を使います。recommended preset は Vue と TypeScript の parser を含み、
Vize のネイティブなファイル単位のルールを `*.vue` に適用します。

```js
// eslint.config.mjs
import vize from "eslint-plugin-vize";

export default [vize.configs.recommended];
```

小さい preset や個別のルールも、同じ flat config に記述します。

```js
// eslint.config.mjs
import { createVizeConfig } from "eslint-plugin-vize";

export default [
  createVizeConfig({ presets: ["essential"] }),
  {
    files: ["**/*.vue"],
    rules: { "vize/a11y/alt-text": "error" },
  },
];
```

このアダプターは、ファイル単位のルールを ESLint から実行します。プロジェクトグラフやファイル間ルールが
必要な場合は `vize lint` または Vite+ の lint タスクを使います。同じ CI ジョブで、同じ Vize ルールを
両方のホストから実行しないでください。

## Prettier

Prettier のファイル探索、ignore ファイル、エディター連携、設定を残したまま、Vize Glyph で Vue SFC と
Musea の `.art` ファイルをフォーマットします。

```json
{
  "plugins": ["prettier-plugin-vize"],
  "printWidth": 100,
  "singleQuote": true
}
```

```bash
prettier --write "src/**/*.{vue,art}"
prettier --check "src/**/*.{vue,art}"
```

同じファイルには、このプラグインか `vize fmt` のどちらか一方を使います。このプラグインは Prettier の
設定と `.prettierignore` に従い、Vize のプロジェクト設定は読み込みません。

## Babel

Babel が JSX / TSX の変換を担当しているプロジェクトで使います。Vite の SFC プラグインとは独立しています。

```bash
npm install --save-dev @babel/core @vizejs/babel-plugin
```

```js
// babel.config.mjs
export default {
  plugins: [["@vizejs/babel-plugin", { jsxMode: "vdom" }]],
  sourceMaps: true,
};
```

Babel で型を取り除く場合は `@babel/preset-typescript` を使います。`jsxMode: "vapor"` は、対応する Vue Vapor
runtime を使うアプリだけで指定してください。意味上の違いは [Babel JSX 互換](./jsx-babel-compat.md)を参照してください。
SSR と抽出した style のオプションはパッケージ README にあります。

## TypeScript Language Service plugin

`@vizejs/typescript-vue-plugin` は、TypeScript 5.7–6.x の JavaScript 版 TypeScript Language Service に Vue の
projection を追加します。二つ目の言語サーバーは起動しません。また VS Code 専用ではありません。
プロジェクトの plugin を読み込む tsserver client なら、同じ `tsconfig.json` の設定を利用できます。

```json
{
  "compilerOptions": {
    "strict": true,
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "plugins": [{ "name": "@vizejs/typescript-vue-plugin" }]
  },
  "include": ["src"]
}
```

plugin の設定を変えたら tsserver を再起動します。VS Code ではワークスペース版 TypeScript を選び、ほかの
client でもプロジェクトの TypeScript server を起動してください。通常の `tsc` は Language Service plugin を
読み込みません。TypeScript 7 の `tsc`、tsgo、tsrs では [Content Mapper](./content-mapper.md)を使います。

## VS Code 以外のエディター

VS Code 拡張機能は便利な導入方法ですが、必須ではありません。`vize lsp` は標準の Language Server Protocol を
stdio で提供するため、Zed、Neovim、Vim、Helix、Emacs、そのほかの LSP client から利用できます。

```bash
npm install --global vize
vize lsp
```

[言語サーバーのガイド](./language-server.md)に各エディターの設定例と、`recommended`・`lint`・`off` の
プロファイルをまとめています。別の Vue 言語サーバーに型チェック・補完・ナビゲーションを任せる場合は、
`lint` プロファイルから始めてください。

## 担当を重複させない

- Parcel が SFC build を担当するときは Parcel transformer を使います。同じ Vue entry を Parcel と Vite の
  両方に渡さないでください。
- 同じ Vize ルールには、`eslint-plugin-vize` またはネイティブの Vize lint コマンドの一方を使います。
- 同じファイルには、`prettier-plugin-vize` または `vize fmt` の一方を使います。
- Vue の主な言語サーバーを一つ選びます。Vue - Official を残す場合は lint のみのプロファイルを使います。
- Babel plugin は Babel が JSX / TSX の変換を担当するときだけ追加します。SFC build、lint、format、
  エディター対応を置き換えるものではありません。
