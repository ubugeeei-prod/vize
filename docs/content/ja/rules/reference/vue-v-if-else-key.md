---
title: "vue/v-if-else-key"
description: "条件分岐で繰り返すコンポーネントに key を指定します。"
---

# `vue/v-if-else-key`

条件分岐で繰り返すコンポーネントに key を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

条件分岐で繰り返す `Panel` に key がありません。

```vue annotate="remove:2"
<template>
  <section><Panel v-if="ready" /><Panel v-else /></section>
</template>
```

## 良い例

各条件コンポーネントに安定した一意の key を指定します。

```vue annotate="add:2"
<template>
  <section><Panel v-if="ready" key="ready" /><Panel v-else key="fallback" /></section>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-if-else-key": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/v_if_else_key.rs#L17)
