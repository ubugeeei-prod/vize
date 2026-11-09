---
title: "vue/quote-props"
description: "不要な引用符付きオブジェクトプロパティを検出します。"
---

# `vue/quote-props`

不要な引用符付きオブジェクトプロパティを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

識別子として使える `name` キーを不要に引用しています。

```vue annotate="remove:2"
<template>
  <Widget :options="{ "name": value }" />
</template>
```

## 良い例

プロパティを引用符なしの識別子で書きます。

```vue annotate="add:2"
<template>
  <Widget :options="{ name: value }" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/quote-props": "warn"
    },
    "ruleOptions": {
      "vue/quote-props": {
        "vue/quote-props": "as-needed"
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L174)
