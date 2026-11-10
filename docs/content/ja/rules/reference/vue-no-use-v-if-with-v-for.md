---
title: "vue/no-use-v-if-with-v-for"
description: "同じ要素での v-if と v-for の併用を検出します。"
---

# `vue/no-use-v-if-with-v-for`

同じ要素での v-if と v-for の併用を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

同じ li に v-if と v-for を指定し、ループの変数で表示条件を検査しています。

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## 良い例

computed で表示する項目を先に絞り込み、テンプレートではその配列の各項目を繰り返し描画します。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-use-v-if-with-v-for": "warn"
    },
    "ruleOptions": {
      "vue/no-use-v-if-with-v-for": {
        "allowUsingIterationVar": true
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

`allowUsingIterationVar` の既定値は `false` です。`true` にすると、同じ要素の `v-if` が
`v-for` の反復変数を参照する場合だけ許可し、無関係な条件は引き続き報告します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35)
