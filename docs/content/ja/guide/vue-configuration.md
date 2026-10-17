---
title: Vue の設定を共有する
---

<!-- Reviewed translation; source: guide/vue-configuration.md -->

# Vue の設定を共有する

ネイティブの Vue 設定を `vite.config.ts` の `vue` にまとめます。コンパイラ、
CLI、エディタで同じオプション名を使えます。ビルド、型チェック、Lint、整形を
まとめて実行するには、[Vite+ 連携](./vite-plus.md)から始めてください。

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
`version`、`vite`、`musea`、`experimentals`、`dialect`、`name`、`entries` も指定できます。コンパイラと整形のセクションには
`enabled` はありません。型チェッカー、Linter、言語サーバーでは、ネイティブと
同じ `enabled` を使えます。未知のキーを canonical な `vue` に置くとエラーになります。

## 既存の Vite+ 設定を使う

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

const vpConfig = { server: { port: 3000 } };
const vueConfig = {
  compiler: { sourceMap: true },
  formatter: { singleQuote: true },
};

export default defineConfig(vpConfig).withVue(vueConfig);
```

既存の Vite+ オブジェクト、Promise、設定関数、async 関数をそのまま渡せます。
設定関数には Vite の環境を渡し、プラグイン、フック、Vite+ のフィールドを保ちます。
Vue プラグインの注入は1回だけ実行します。通常の JavaScript オブジェクトを import して
共有できるため、Vize 専用の設定ファイルや独自の `extends` は不要です。

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

canonical な entry に置けるのは、現在 `basePath`、`files`、`ignores`、`linter` だけです。
ほかのセクションと `version` はプロジェクト全体に指定し、entry に置くとエラーになります。
CLI のスコープ付き Lint は従来の動作を保ちます。エディタでのファイル単位のスコープ適用と、
ほかのセクションのスコープ対応は未完了です。

## TypeScript プロジェクトの設定

TypeScript の `compilerOptions`、標準の `extends`、`references` は既存の
TypeScript プロジェクトに置きます。`strictTemplates` などの Vue テンプレート設定は
`vueCompilerOptions` に記述します。CLI とエディタはこのプロジェクト設定を使います。
[設定ガイド](./configuration.md#typescript-and-vue-template-checking)も参照してください。

## 既存の設定を移行する

トップレベルの `vize:` と `vize.config.*` のネイティブ設定を `vue:` に移します。
Vue ツールには `typeChecker`、`linter`、`formatter` を使い、Vite+ のタスク拡張には
`typecheck`、`lint.vue`、`fmt.vue`、`pack.vue` を使います。廃止した専用設定の形式と
Vize 独自の `extends` は読みません。共有する設定は通常の JavaScript オブジェクトとして
import してください。`vueCompilerOptions` を含む TypeScript の設定は
`tsconfig.json` に置きます。
