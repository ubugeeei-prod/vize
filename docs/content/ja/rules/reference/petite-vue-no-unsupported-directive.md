---
title: "petite-vue/no-unsupported-directive"
description: "petite-vue が対応しないディレクティブを検出します。"
---

# `petite-vue/no-unsupported-directive`

petite-vue が対応しないディレクティブを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | なし | なし | なし |

適用範囲：petite-vue と判定された HTML 文書。通常の Vue SFC は対象外です。

## 悪い例

`v-memo`、`v-slot:header`、カスタムの `v-my-directive` は petite-vue の対応ディレクティブ一覧にありません。petite-vue の script によって、この HTML が対象の方言として判定されます。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 良い例

対応している `v-scope`、`v-effect`、`v-if`、`v-bind`、`v-on` を使い、未対応のディレクティブへの依存を取り除きます。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [petite-vue ルール](../petite-vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43)
