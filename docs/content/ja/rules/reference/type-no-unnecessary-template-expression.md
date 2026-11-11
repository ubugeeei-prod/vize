---
title: "type/no-unnecessary-template-expression"
description: "単一の式を不要に包むテンプレートリテラルを検出します。"
---

# `type/no-unnecessary-template-expression`

単一の式を不要に包むテンプレートリテラルを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

テンプレートには label しかないため、新しい文字列を組み立てていません。

```vue annotate="remove:3"
<script setup lang="ts">
const label = "a";
const wrapped = `${label}`;
</script>
```

## 良い例

label を代入すると、テンプレートで包まずに文字列を使います。

```vue annotate="add:3"
<script setup lang="ts">
const label = "a";
const wrapped = label;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-template-expression": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_template_expression.rs#L5)
