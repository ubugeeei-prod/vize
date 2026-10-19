---
title: "css/prefer-nested-selectors"
---

# `css/prefer-nested-selectors`

子孫セレクターを CSS のネスト構文でまとめます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

ルールをネストせず、子孫セレクター `.card .title` に親のセレクターも含めて指定しています。

```vue
<style scoped>
.card .title { color: red; }
</style>
```

## 良い

`.title` のルールを `.card` の中にネストし、親子のスタイルの関係を一か所にまとめます。

```vue
<style scoped>
.card { .title { color: red; } }
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [全ルール](../all.md)
