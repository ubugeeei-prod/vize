---
title: "type/no-array-delete"
description: "配列から要素を delete している箇所を検出します。"
---

# `type/no-array-delete`

配列から要素を delete している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

delete numbers[0] は配列要素を消し、詰め直さずに穴を残します。

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
delete numbers[0];
</script>
```

## 良い例

splice(0, 1) は先頭要素を削除し、配列を密なまま保ちます。

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.splice(0, 1);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-array-delete": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_array_delete.rs#L5)
