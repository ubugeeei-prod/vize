---
title: "vue/attribute-hyphenation"
description: "コンポーネントの prop 属性名を設定した形式に揃えます。"
---

# `vue/attribute-hyphenation`

コンポーネントの prop 属性名を設定した形式に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

コンポーネントの属性名を camelCase の firstName にしています。

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

## 良い例

first-name に変更し、設定されたハイフン区切りの属性名の方針に合わせます。

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
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
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L32)
