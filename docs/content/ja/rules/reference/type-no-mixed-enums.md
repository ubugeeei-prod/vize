---
title: "type/no-mixed-enums"
description: "文字列と数値の列挙メンバーが混在する箇所を検出します。"
---

# `type/no-mixed-enums`

文字列と数値の列挙メンバーが混在する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

列挙は数値メンバーと文字列メンバーを混ぜています。

```vue annotate="remove:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = "b",
}
</script>
```

## 良い例

両方のメンバーが数値なので、列挙の値の領域は一つです。

```vue annotate="add:4"
<script setup lang="ts">
enum Mixed {
  A = 0,
  B = 1,
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-mixed-enums": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_mixed_enums.rs#L5)
