---
title: "vue/use-v-on-exact"
---

# `vue/use-v-on-exact`

modifier 付きのイベント操作と競合する handler に .exact を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `essential`, `nuxt`, `opinionated`  
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
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

通常の click handler も Ctrl-click で動くため、別の `.ctrl` handler と重複します。

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## 良い

`.exact` で通常の handler を修飾キーのない click に限定し、Ctrl 専用の handler と分けます。

```vue
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [全ルール](../all.md)
