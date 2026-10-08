---
title: "vue/require-toggle-inside-transition"
---

# `vue/require-toggle-inside-transition`

transition の子要素に表示を切り替える条件を指定します。

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
        "vue/require-toggle-inside-transition": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`<Transition>` 内の静的な子に表示の切り替えや動的な選択がなく、enter / leave が発生する条件がありません。

```vue
<template>
<transition>
<div>content</div>
</transition>
</template>
```

## 良い

`v-if="show"` で子の有無を切り替え、enter / leave の対象にします。

```vue
<template>
<transition>
<div v-if="show">content</div>
</transition>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [全ルール](../all.md)
