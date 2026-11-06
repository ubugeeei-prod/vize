---
title: "css/require-font-display"
description: "@font-face に font-display を指定します。"
---

# `css/require-font-display`

@font-face に font-display を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

font-face にフォントの参照先はありますが、font-display の方針を指定していません。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

## 良い例

font-display: swap を追加し、代替フォントで表示した後、読み込まれたフォントへ切り替える方針を指定します。

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [CSS ルール](../css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13)
