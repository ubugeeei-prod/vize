---
title: "css/no-important"
description: "通常のカスケードを上書きする !important を検出します。"
---

# `css/no-important`

通常のカスケードを上書きする !important を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

color に `!important` を付け、通常のカスケードの優先順位を上書きしています。

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

## 良い例

important を使わず、カスタムプロパティから色を参照します。

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-important": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [CSS ルール](../css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14)
