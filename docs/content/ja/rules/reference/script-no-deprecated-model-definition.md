---
title: "script/no-deprecated-model-definition"
description: "Vue 3 で削除された model オプションを検出します。"
---

# `script/no-deprecated-model-definition`

Vue 3 で削除された model オプションを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

コンポーネントが Vue 2 の `model` オプションを `prop` と `event` 付きで宣言しています。

```vue annotate="remove:3,4,5,6"
<script lang="ts">
export default {
  model: {
    prop: 'value',
    event: 'input'
  }
}
</script>
```

## 良い例

`modelValue` prop と `update:modelValue` イベントを宣言します。

```vue annotate="add:3,4"
<script lang="ts">
export default {
  props: ['modelValue'],
  emits: ['update:modelValue']
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-model-definition": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_model_definition.rs#L32)
