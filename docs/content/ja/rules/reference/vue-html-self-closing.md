---
title: "vue/html-self-closing"
description: "要素の種類ごとに自己終了タグの形式を揃えます。"
---

# `vue/html-self-closing`

要素の種類ごとに自己終了タグの形式を揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | あり | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

空のコンポーネントと通常の div を開始タグと終了タグの組で書いており、設定された自己終了タグの表記に合いません。

```vue annotate="remove:3,4"
<template>
  <section>
    <MyComponent></MyComponent>
    <div></div>
  </section>
</template>
```

## 良い例

空のコンポーネントと通常要素は自己終了タグにし、void 要素はスラッシュを付けません。内容がある要素は終了タグを残します。

```vue annotate="add:3,4,5,6,7"
<template>
  <section>
    <MyComponent />
    <div />
    <img>
    <br>
    <div>content</div>
  </section>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-self-closing": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L31)
