---
title: "vue/no-useless-template-attributes"
---

# `vue/no-useless-template-attributes`

template 要素の効果がない属性を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

条件付きの template に class を指定していますが、この構造用の wrapper は DOM 要素を表示しません。

```vue
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

## 良い

実際に表示する p に class を移し、構造を指定する template の v-if は残します。

```vue
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [全ルール](../all.md)
