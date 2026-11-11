---
title: "type/no-for-in-array"
description: "配列に対する for-in を検出します。"
---

# `type/no-for-in-array`

配列に対する for-in を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

for-in は配列の要素値ではなくキーをたどります。

```vue annotate="remove:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const key in numbers) {
    console.log(key);
  }
}
</script>
```

## 良い例

for-of は配列の数値要素をたどります。

```vue annotate="add:4,5"
<script setup lang="ts">
const numbers = [1, 2, 3];
function loopsKeys(): void {
  for (const n of numbers) {
    console.log(n);
  }
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
      "type/no-for-in-array": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_for_in_array.rs#L5)
