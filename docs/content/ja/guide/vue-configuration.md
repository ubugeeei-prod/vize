---
title: Vue の設定を共有する
---

<!-- Reviewed translation; source: guide/vue-configuration.md -->

# Vue の設定を共有する

ネイティブの Vue 設定を `vite.config.ts` の `vue` にまとめます。コンパイラ、
CLI、エディターで同じオプション名を使えます。ビルド、型チェック、lint、整形を
まとめて実行するには、[Vite+ 連携](./vite-plus.md)から始めてください。

この API は次のマイナーリリースに向けた変更です。現在公開されているバージョンでは、
[設定ガイド](./configuration.md)にある方法を使ってください。

## 設定を一つにまとめる

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    version: "3",
    compiler: { sourceMap: true },
    typeChecker: { strict: true },
    linter: { preset: "essential" },
    formatter: { singleQuote: true },
    languageServer: { hover: true },
  },
  server: { port: 3000 },
});
```

`vue` では、ネイティブの `compiler`、`typeChecker`、`linter`、`formatter`、
`languageServer`、`globalTypes`、`files`、`ignores`、`basePath`、`lib` を使います。
`version` と `entries` も指定できます。コンパイラと整形のセクションには
`enabled` はありません。型チェッカー、リンター、言語サーバーでは、ネイティブと
同じ `enabled` を使えます。`vue` に未対応のキーを指定するとエラーになります。

## 既存の Vite+ 設定を使う

```ts
import { defineConfig } from "vite-plus";
import { withVue } from "@vizejs/vite-plugin/vite-plus";
import vpConfig from "./vite.base.ts";

export default withVue(
  { compiler: { sourceMap: true }, formatter: { singleQuote: true } },
  defineConfig(vpConfig),
);
```

最小の形は `withVue({}, defineConfig(vpConfig))` です。第1引数に Vue の設定、
第2引数に既存の Vite+ 設定を渡します。オブジェクト、Promise、設定関数、async 関数を
使えます。既存の設定関数には、評価時に Vite の環境が渡されます。既存のプラグイン、
フック、Vite+ のフィールドを保ち、Vite+ のプラグイン注入は1回だけ実行します。

## パスとスコープ

Vite の設定にあるネイティブの相対パスは、選択された Vite の `root` を基準にします。
`root` 自体は設定ファイルのディレクトリから解決します。CLI に直接渡したファイル名は、
従来どおりコマンドを実行したディレクトリを基準にします。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  root: "packages/app",
  vue: {
    files: ["src/**/*.vue"],
    entries: [
      {
        files: ["src/trusted/**/*.vue"],
        linter: { rules: { "vue/no-v-html": "off" } },
      },
    ],
  },
});
```

`vue.entries` の各エントリに指定できるのは、現在 `basePath`、`files`、`ignores`、`linter` だけです。
ほかのセクションと `version` はプロジェクト全体に指定し、エントリに置くとエラーになります。
CLI のスコープ別の lint は従来どおり動作します。エディターでのファイル単位のスコープ適用と、
ほかのセクションのスコープ対応は未完了です。

## 既存の設定を移行する

トップレベルの `vize` にあるネイティブ設定を `vue` に移します。
Vite+ のタスク用の `typecheck`、`lint.vize`、`fmt.vize` の代わりに、
`typeChecker`、`linter`、`formatter` を使います。既存の設定の入口も引き続き受け付けます。
従来の設定を第1引数に取る `withVue(config)` は、Vue オプションを先に渡す形に変わります。
以前の呼び出し方を保つ場合は、`defineConfig(config)` または `withVize(config)` を使ってください。
読み込む Vite 設定に従来の形式と `vue` の両方がある場合、`vue` に明示した値を優先します。
省略した値は継承し、配列は置き換え、`false` はそのまま保ちます。

ネイティブ CLI とエディターの自動探索では、既存の `vize.config.*` が引き続き優先されます。
Vite+ ヘルパーは選択された設定を使い、選択されていない専用ファイルの値を混ぜません。
`vue` の設定をネイティブコマンドと共有する場合は、読み込まれる Vite 設定にまとめてください。
既存の形式と明示的なファイル選択は、[専用設定のリファレンス](./configuration-reference.md)を
参照してください。
