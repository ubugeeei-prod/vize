---
title: "vue/no-negated-v-if-condition"
---

# `vue/no-negated-v-if-condition`

v-else がある条件分岐の否定条件を反転して読みやすくします。

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
        "vue/no-negated-v-if-condition": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

v-else と対になる v-if を、否定した条件で始めています。

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

## 良い

先に正の ok の条件を使います。条件を反転する際は分岐の内容も入れ替えます。単独の否定の v-if や !== の比較は許可されます。

```vue
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [全ルール](../all.md)
