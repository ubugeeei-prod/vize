---
title: "vue/a11y-img-alt"
---

# `vue/a11y-img-alt`

画像に代替テキストの alt 属性を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/a11y-img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

静的な画像にも src をバインドする画像にも alt がありません。

```vue
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

## 良い

情報のある画像には説明、装飾には空の alt、動的な画像には説明のバインディングを指定します。

```vue
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [全ルール](../all.md)
