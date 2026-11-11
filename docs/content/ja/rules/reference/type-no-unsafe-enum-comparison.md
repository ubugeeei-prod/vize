---
title: "type/no-unsafe-enum-comparison"
description: "列挙値を別の値域と比較している箇所を検出します。"
---

# `type/no-unsafe-enum-comparison`

列挙値を別の値域と比較している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

value は数値なので、列挙メンバー Color.Red との比較は値の領域をまたぎます。

```vue annotate="remove:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
function isRed(value: number): boolean {
  return value === Color.Red;
}
</script>
```

## 良い例

value は Color なので、比較は Color 列挙の中で完結します。

```vue annotate="add:6"
<script setup lang="ts">
enum Color {
  Red = 0,
  Green = 1,
}
function isRed(value: Color): boolean {
  return value === Color.Red;
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
      "type/no-unsafe-enum-comparison": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_enum_comparison.rs#L5)
