---
title: "vue/valid-v-model"
description: "v-model の値・引数・修飾子を検査します。"
---

# `vue/valid-v-model`

v-model の値・引数・修飾子を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

ネイティブの `<div>` にはフォーム要素として `v-model` を使えません。また、input に値なしで指定したディレクティブには、書き込み先の式がありません。

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

## 良い例

input、select、textarea、カスタムコンポーネントを例の書き込み可能な変数にバインドします。

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-model": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36)
