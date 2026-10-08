---
title: "petite-vue/no-unsupported-directive"
---

# `petite-vue/no-unsupported-directive`

petite-vue が対応しないディレクティブを検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`v-memo`、`v-slot:header`、カスタムの `v-my-directive` は petite-vue の対応ディレクティブ一覧にありません。petite-vue の script によって、この HTML が対象の方言として判定されます。

```html
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 良い

対応している `v-scope`、`v-effect`、`v-if`、`v-bind`、`v-on` を使い、未対応のディレクティブへの依存を取り除きます。

```html
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [全ルール](../all.md)
