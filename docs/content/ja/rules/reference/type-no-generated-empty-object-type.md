---
title: "type/no-generated-empty-object-type"
description: "型操作で空のオブジェクト型が生成される箇所を検出します。"
---

# `type/no-generated-empty-object-type`

型操作で空のオブジェクト型が生成される箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

Omit が唯一のキーを除くため、別名は空のオブジェクト型になります。

```vue annotate="remove:2"
<script setup lang="ts">
type Emptied = Omit<{ a: number }, "a">;
</script>
```

## 良い例

Omit は a を除いても b を残すため、別名は空のオブジェクト型になりません。

```vue annotate="add:2"
<script setup lang="ts">
type Kept = Omit<{ a: number; b: string }, "a">;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-generated-empty-object-type": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_generated_empty_object_type.rs#L5)
