---
title: "vue/a11y-img-alt"
description: "画像に代替テキストの alt 属性を指定します。"
---

# `vue/a11y-img-alt`

画像に代替テキストの alt 属性を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

静的な画像にも src をバインドする画像にも alt がありません。

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

## 良い例

情報を伝える画像には内容を説明する alt テキスト、装飾画像には空の alt を指定します。動的な画像には説明をバインドします。

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
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
        "vue/a11y-img-alt": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33)
