---
title: "vue/require-component-is"
description: "動的 component 要素に :is を指定します。"
---

# `vue/require-component-is`

動的 component 要素に :is を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

動的な `<component>` に `is` がなく、描画するコンポーネントを選べません。

```vue annotate="remove:2"
<template>
  <component />
</template>
```

## 良い例

`:is="currentComponent"` で描画対象を指定します。対象は実行時に変更できます。

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-is": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27)
