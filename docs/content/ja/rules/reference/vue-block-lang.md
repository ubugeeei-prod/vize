---
title: "vue/block-lang"
description: "SFC ブロックの lang を設定した言語に揃えます。"
---

# `vue/block-lang`

SFC ブロックの lang を設定した言語に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

script ブロックに設定した lang 属性がありません。

```vue annotate="remove:1"
<script setup>
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## 良い例

script ブロックの lang を設定した言語にします。

```vue annotate="add:1"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/block-lang": "warn"
    },
    "ruleOptions": {
      "vue/block-lang": {
        "script": {
          "lang": "ts",
          "allowNoLang": false
        }
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/block_lang.rs#L12)
