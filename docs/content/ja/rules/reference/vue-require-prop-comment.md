---
title: "vue/require-prop-comment"
description: "宣言した各 prop に JSDoc コメントを付けます。"
---

# `vue/require-prop-comment`

宣言した各 prop に JSDoc コメントを付けます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

size プロパティに JSDoc コメントがありません。

```vue annotate="remove:2"
<script setup lang="ts">
const { size } = defineProps<{ size: string }>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## 良い例

size プロパティに JSDoc コメントを付けます。

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
const { size } = defineProps<{
  /** The rendered size. */
  size: string;
}>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-prop-comment": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_prop_comment.rs#L19)
