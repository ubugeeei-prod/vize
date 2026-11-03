---
title: "html/no-dupe-style-properties"
description: "静的 style 属性内のプロパティ重複を検出します。"
---

# `html/no-dupe-style-properties`

静的 style 属性内のプロパティ重複を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

静的な style の中で同じプロパティを重複させています。margin と MARGIN も同じプロパティとして扱われます。

```vue annotate="remove:2,3"
<template>
  <div style="color: red; color: blue">text</div>
  <div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

## 良い例

静的な style 属性には color と background をそれぞれ一度だけ指定し、同じプロパティの重複を避けます。動的な style バインディングはこの静的属性の検査の対象外です。

```vue annotate="add:2,3"
<template>
  <div style="color: red; background: blue">text</div>
  <div :style="{ color: a, color: b }">text</div>
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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37)
