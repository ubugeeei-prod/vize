---
title: "css/prefer-nested-selectors"
description: "子孫セレクターを CSS のネスト構文でまとめます。"
---

# `css/prefer-nested-selectors`

子孫セレクターを CSS のネスト構文でまとめます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

ルールをネストせず、子孫セレクター `.card .title` に親のセレクターも含めて指定しています。

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

## 良い例

`.title` のルールを `.card` の中にネストし、親子のスタイルの関係を一か所にまとめます。

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/prefer-nested-selectors": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [CSS ルール](../css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14)
