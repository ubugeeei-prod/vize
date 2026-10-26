---
title: 実験的な Router の型
description: 公開済みの Vue Router の実験的な型を、型チェックとエディターで明示的に有効にします。
---

# 実験的な Router の型

Vize は Vue Router のプロジェクト単位の typed-router 設定を認識します。
`vize check`、content mapper、Vize の言語サーバーで、同じ TypeScript プロジェクトに
Router の生成宣言と opt-in を含めてください。

この手順の対象は **`vue-router@5.4.0`** です。実験的な API はバージョン間で変わります。
[upstream のガイド](https://router.vuejs.org/experimental/router-resolver)には、
公開済みパッケージにまだ入っていない変更が載る場合があります。

## ルートの型宣言を生成する

Vue コンパイラーの前に Vue Router のスキャナーを設定します。

```ts
// vite.config.ts
import { defineConfig } from "vite";
import VueRouter from "vue-router/vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    VueRouter({
      routesFolder: "src/pages",
      dts: "typed-router.d.ts",
      experimental: { paramParsers: { dir: "src/params" } },
    }),
    vize(),
  ],
});
```

Vite を一度起動すると `typed-router.d.ts` が生成されます。Vite の起動前に
型チェックする CI では、公開済みの同じスキャナーを直接実行できます。

```js
// scripts/generate-routes.mjs
import { createRoutesContext, resolveOptions } from "vue-router/unplugin";

const context = createRoutesContext(
  resolveOptions({
    root: process.cwd(),
    routesFolder: "src/pages",
    dts: "typed-router.d.ts",
    experimental: { paramParsers: { dir: "src/params" } },
  }),
);
try {
  await context.scanPages(false);
  context.writeConfigFiles();
} finally {
  context.stopWatcher();
}
```

```bash
node scripts/generate-routes.mjs
```

生成した宣言をコミットするか、型チェック前に生成してください。ページのパスや
パーサーの宣言を変更したら再生成します。Vize はこの宣言を読み込みます。
ルートのスキャン自体は Vue Router が行います。

## プロジェクトごとに有効にする

ページを含む実際のプロジェクトに、次の設定を追加します。既存の
`compilerOptions`、`types`、プラグインは残してください。

```json
{
  "compilerOptions": {
    "target": "ESNext",
    "module": "ESNext",
    "moduleResolution": "Bundler",
    "strict": true,
    "noEmit": true,
    "skipLibCheck": true,
    "rootDir": ".",
    "types": ["vue-router/auto-routes"]
  },
  "vueCompilerOptions": {
    "plugins": ["vue-router/volar/sfc-typed-router"]
  },
  "include": ["typed-router.d.ts", "src/**/*.ts", "src/**/*.vue"]
}
```

このプラグイン名により、Vize 組み込みの typed-router 変換が有効になります。
任意の Volar プラグインを実行する設定ではありません。指定を削除すると変換は
無効になります。モノレポでは、あるプロジェクトだけで有効にできます。

この公開バージョンの手順では、Router の実験的な型宣言との互換性のために
`skipLibCheck: true` を使います。作成したスクリプトやテンプレートは引き続き検査します。

## 実験的な Router の型を登録する

```ts
// src/router/index.ts
import { createWebHistory } from "vue-router";
import { experimental_createRouter as createRouter } from "vue-router/experimental";
import { resolver } from "vue-router/auto-resolver";

export const router = createRouter({ history: createWebHistory(), resolver });

declare module "vue-router" {
  interface TypesConfig {
    Router: typeof router;
  }
}
```

実験的な Router を使う場合は、upstream のセットアップ手順に従い、`vue-router`
の `RouterLink` と `RouterView` を Vue アプリに登録してください。

## パース済みの値を使う

カスタムパーサーのモジュールでは `parser` を export します。

```ts
// src/params/date.ts
import { defineParamParser, miss } from "vue-router/experimental";

export const parser = defineParamParser<Date>({
  get(value) {
    const date = new Date(value);
    if (Number.isNaN(date.getTime())) miss();
    return date;
  },
  set: (value) => value.toISOString(),
});
```

```vue
<!-- src/pages/products/[id=int].vue -->
<script setup lang="ts">
import { useRoute } from "vue-router";
import { definePage } from "vue-router/experimental";

definePage({
  params: {
    query: {
      page: { parser: "int", format: "value", default: 1 },
      at: { parser: "date", format: "value" },
    },
  },
});

const route = useRoute();
const id: number = route.params.id;
const page: number = route.params.page;
const at: Date | undefined = route.params.at;
</script>

<template>{{ id.toFixed() }} / {{ page.toFixed() }} / {{ at?.getFullYear() }}</template>
```

公開済みの 5.4.0 では、`useRoute` と `useRouter` は `vue-router` から、
`definePage` とパーサーのヘルパーは `vue-router/experimental` から import します。

## 同じプロジェクトを選ぶ

```bash
vize check --tsconfig tsconfig.app.json
tsc --runExternalCode --noEmit -p tsconfig.app.json
```

2 つ目のコマンドには、[content mapper の設定](./content-mapper.md)と、
マッパー対応のコンパイラーも必要です。どちらも `typed-router.d.ts` と上の
opt-in を含むプロジェクトを指定してください。言語サーバーでも同じプロジェクトを開き、
recommended プロファイルで型チェックを有効にします。`tsconfig.json` 以外を使う場合は、
Vite の設定の `vue.typeChecker.tsconfig` に同じファイルを指定してください。
[設定リファレンス](./configuration-reference.md#静的解析のオプション)も参照してください。

VS Code と Art の拡張機能も、この言語サーバーのプロジェクト設定を使います。
拡張機能専用の experimental-router 設定はありません。

## 公開バージョンの範囲

5.4.0 パッケージは `_RoutePropsForFilePath` を export しておらず、`props: true`
から `defineProps()` を自動で拡張する新しい動作も含んでいません。未公開のヘルパー宣言を
プロジェクトにコピーして有効化しないでください。新しい upstream の契約は、実際の
対応パッケージと別の互換性検証が必要です。

追加のスクリプト・テンプレート言語における実験的なルートの型は、別途検証しています。
この手順の対象は TypeScript の script と HTML の template を使う Vue SFC です。
すべてのプリプロセッサーでの同等動作を保証するものではありません。
