---
title: "vue/use-unique-element-ids"
---

# `vue/use-unique-element-ids`

静的 ID の代わりに useId() で再利用可能な ID を生成します。

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
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

固定の `email` ID がコンポーネントの各インスタンスで重複し、複数表示時に label の参照先が曖昧になります。

```vue
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

## 良い

`useId()` の `emailId` を label の `for` と input の `id` の両方に binding します。

```vue
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全ルール](../all.md)
