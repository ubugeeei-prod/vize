---
title: "vapor/no-vue-lifecycle-events"
description: "Vapor では未対応の、要素ごとに指定する @vue:* ライフサイクルイベントを検出します。"
---

# `vapor/no-vue-lifecycle-events`

Vapor では未対応の、要素ごとに指定する @vue:* ライフサイクルイベントを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

input にテンプレートの @vue:mounted を指定しています。

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

## 良い例

script の onMounted からテンプレートの ref を参照し、input にフォーカスします。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/no-vue-lifecycle-events": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vapor ルール](../vapor.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34)
