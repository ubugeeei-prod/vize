---
title: "vue/no-deprecated-v-on-number-modifiers"
---

# `vue/no-deprecated-v-on-number-modifiers`

v-on の削除済み数値 keyCode modifier を検出します。

既定の重大度: `error`  
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
        "vue/no-deprecated-v-on-number-modifiers": "error"
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
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## 良い

```vue
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [全ルール](../all.md)
