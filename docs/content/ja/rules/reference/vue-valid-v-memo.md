---
title: "vue/valid-v-memo"
---

# `vue/valid-v-memo`

v-memo の値を配列の式にします。

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
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

値のない `v-memo` では、サブツリーを再利用する判断に必要な依存式を渡せません。

```vue
<template>
  <div v-memo></div>
</template>
```

## 良い

`v-memo="[valueA, valueB]"` でメモ化に使う依存配列を渡します。

```vue
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [全ルール](../all.md)
