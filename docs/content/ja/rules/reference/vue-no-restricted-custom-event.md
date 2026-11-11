---
title: "vue/no-restricted-custom-event"
description: "設定したカスタムイベントを使う箇所を検出します。"
---

# `vue/no-restricted-custom-event`

設定したカスタムイベントを使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートが禁止された legacyEvent を emit しています。

```vue annotate="remove:2"
<template>
  <button type="button" @click="$emit('legacyEvent')">x</button>
</template>
```

## 良い例

テンプレートは save を emit します。

```vue annotate="add:2"
<template>
  <button type="button" @click="$emit('save')">x</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-custom-event": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-custom-event": [
        "legacyEvent"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_custom_event.rs#L19)
