---
title: "vue/single-style-block"
---

# `vue/single-style-block`

SFC の style を一つのブロックにまとめます。

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "vue/single-style-block": "warn"
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
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

## 良い

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [全ルール](../all.md)
