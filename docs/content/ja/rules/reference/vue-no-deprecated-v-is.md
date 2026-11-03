---
title: "vue/no-deprecated-v-is"
description: "非推奨の v-is ディレクティブを検出します。"
---

# `vue/no-deprecated-v-is`

非推奨の v-is ディレクティブを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

Vue 3.1 で非推奨になった `v-is` で、描画するコンポーネントを切り替えています。

```vue annotate="remove:2"
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## 良い例

`vue:` プレフィックス付きの `is` 属性、または `<component :is>` で切り替えます。

```vue annotate="add:2,3"
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
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
        "vue/no-deprecated-v-is": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27)
