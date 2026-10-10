---
title: "vue/no-root-v-if"
---

# `vue/no-root-v-if`

テンプレートの単一ルートに v-if を指定する箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `nuxt`, `opinionated`  
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
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

コンポーネントのルート自体を v-if で表示・非表示にしています。

```vue
<template>
  <div v-if="show">content</div>
</template>
```

## 良い

外側の div をルートとして残し、内側の p に表示条件を指定します。

```vue
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [全ルール](../all.md)
