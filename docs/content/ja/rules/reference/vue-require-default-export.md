---
title: "vue/require-default-export"
description: "コンポーネントの script をデフォルトエクスポートにします。"
---

# `vue/require-default-export`

コンポーネントの script をデフォルトエクスポートにします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントの script にデフォルトエクスポートがありません。

```vue
<script lang="ts">
const label = "a";
</script>
<template>
  <p>label</p>
</template>
```

## 良い例

コンポーネントを script のデフォルトエクスポートにします。

```vue annotate="add:3"
<script lang="ts">
const label = "a";
export default { name: "Label" };
</script>
<template>
  <p>label</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-default-export": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_default_export.rs#L14)
