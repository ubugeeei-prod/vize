---
title: "html/deprecated-element"
description: "非推奨の HTML 要素を検出します。"
---

# `html/deprecated-element`

非推奨の HTML 要素を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

中央揃えのために、非推奨の center 要素を使っています。

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

## 良い例

内容は残し、section とスタイル用のクラスに置き換えます。

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [HTML ルール](../html.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33)
