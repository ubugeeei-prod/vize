---
title: "vue/require-component-registration"
---

# `vue/require-component-registration`

使用するコンポーネントを import または登録します。

既定の重大度: `warning`  
プリセット: `opinionated`  
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
        "vue/require-component-registration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<script setup lang="ts">
// MyButton is never imported.
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

## 良い

```vue
<script setup lang="ts">
import MyButton from "./MyButton.vue";
</script>

<template>
  <MyButton>Save</MyButton>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [全ルール](../all.md)
