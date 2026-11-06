---
title: "css/prefer-logical-properties"
description: "書字方向に対応する CSS の論理プロパティを使います。"
---

# `css/prefer-logical-properties`

書字方向に対応する CSS の論理プロパティを使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

margin-left は文字の方向に関係なく物理的な左側を指定します。

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

## 良い例

margin-inline-start を使い、インライン方向の開始側に余白を指定します。

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
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
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [CSS ルール](../css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15)
