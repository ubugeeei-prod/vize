---
title: "vue/no-shadow-native-events"
description: "ネイティブイベントと衝突するコンポーネントイベント名を検出します。"
---

# `vue/no-shadow-native-events`

ネイティブイベントと衝突するコンポーネントイベント名を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントがネイティブイベントと衝突する `click` を宣言しています。

```vue annotate="remove:2"
<script setup>
const emit = defineEmits(['click'])
</script>
<template />
```

## 良い例

コンポーネント固有の `activate` イベントで衝突を避けます。

```vue annotate="add:2"
<script setup>
const emit = defineEmits(['activate'])
</script>
<template />
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-shadow-native-events": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/events.rs#L27)
