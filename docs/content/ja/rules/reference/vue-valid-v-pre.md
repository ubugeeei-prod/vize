---
title: "vue/valid-v-pre"
description: "v-pre の引数・値・修飾子を検査します。"
---

# `vue/valid-v-pre`

v-pre の引数・値・修飾子を検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンパイルを省く印である `v-pre` に、引数・修飾子・値を渡しています。

```vue annotate="remove:2,3,4"
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## 良い例

値のない `v-pre` でサブツリーをコンパイル対象から外し、マスタッシュをそのまま表示します。

```vue annotate="add:2"
<template>
<div v-pre>{{ raw }}</div>
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
        "vue/valid-v-pre": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L30)
