---
title: "script/consistent-props-emits-key-quotes"
description: "props / emits 宣言で引用符が必須のキーがある場合、すべてのキーを引用符付きに揃えます。"
---

# `script/consistent-props-emits-key-quotes`

props / emits 宣言で引用符が必須のキーがある場合、すべてのキーを引用符付きに揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## Formatter との併用

この lint ルールが宣言単位の引用符を管理します。ルールが必要とする引用符を整形時に削除しないよう、Prettier・Oxfmt・Vize formatter の `quoteProps` を `"preserve"` に設定してください。

## 悪い例

emits 宣言で、引用符が必須の `update:modelValue` キーと、引用符のない `close` キーが混在しています。

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  close: []
}>()
</script>
```

## 良い例

`update:modelValue` は引用符なしの JavaScript 識別子として書けないため、宣言内のすべての静的キーを引用符付きにそろえます。

```vue annotate="add:4,6,7,8,9"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  "close": []
}>()
const props = defineProps({
  "aria-label": String,
  "disabled": Boolean,
})
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-props-emits-key-quotes": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/consistent_key_quotes.rs#L53)
