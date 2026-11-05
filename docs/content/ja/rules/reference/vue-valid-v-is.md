---
title: "vue/valid-v-is"
description: "v-is の適用先・引数・値を検査します。"
---

# `vue/valid-v-is`

v-is の適用先・引数・値を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`v-is` に引数や修飾子を渡す、値がない、ネイティブの HTML 要素ではなくコンポーネントに指定している、といった不正な書き方です。

```vue annotate="remove:2,3,4,5"
<template>
<tr v-is:arg="'MyRow'"></tr>
<tr v-is.mod="'MyRow'"></tr>
<tr v-is></tr>
<MyComponent v-is="'MyRow'" />
</template>
```

## 良い例

`<tr>` などのネイティブ要素に値付きの `v-is` を指定し、代わりに描画するコンポーネントを示します。

```vue annotate="add:2"
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-is": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30)
