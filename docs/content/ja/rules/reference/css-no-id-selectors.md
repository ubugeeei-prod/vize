---
title: "css/no-id-selectors"
description: "詳細度が高い CSS の ID セレクターを検出します。"
---

# `css/no-id-selectors`

詳細度が高い CSS の ID セレクターを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

`#submit` で ID セレクターにスタイルを結び付けています。

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

## 良い例

再利用できる `.submit` のクラスを使い、ID セレクターを取り除きます。

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
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
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [CSS ルール](../css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19)
