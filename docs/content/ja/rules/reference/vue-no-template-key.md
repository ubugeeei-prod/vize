---
title: "vue/no-template-key"
description: "v-for 用ではない template の key 指定を検出します。"
---

# `vue/no-template-key`

v-for 用ではない template の key 指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

繰り返しではない template に key を指定しています。

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

## 良い例

template の v-for に key を付け、繰り返す各フラグメントを識別します。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-key": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L29)
