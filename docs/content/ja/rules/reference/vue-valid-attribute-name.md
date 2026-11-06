---
title: "vue/valid-attribute-name"
description: "有効な属性名を指定します。"
---

# `vue/valid-attribute-name`

有効な属性名を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

悪い例での診断：`parser/template`

不正な属性名は、この防御的なルールに届く前に parser/template で検出されます。悪い例で確認するのは parser/template の検出で、vue/valid-attribute-name が別に出るとは限りません。

## 悪い例

`my"attr` の引用符で属性名が壊れています。この例の診断は `parser/template` で、別のルール診断が出ることを示すものではありません。

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

## 良い例

`my-attr` は正しい属性名で、テンプレートパーサーが属性と値を読み取れます。

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-attribute-name": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27)
