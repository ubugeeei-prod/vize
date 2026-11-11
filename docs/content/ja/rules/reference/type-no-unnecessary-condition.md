---
title: "type/no-unnecessary-condition"
description: "常に真、常に偽、または常に nullish な条件を検出します。"
---

# `type/no-unnecessary-condition`

常に真、常に偽、または常に nullish な条件を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

value は文字列なので nullish にならず、?? の代替値は実行されません。

```vue annotate="remove:2"
<script setup lang="ts">
function orDefault(value: string): string {
  return value ?? "x";
}
</script>
```

## 良い例

value は undefined の可能性があるため、欠けているときに ?? が文字列を補います。

```vue annotate="add:2"
<script setup lang="ts">
function orDefault(value: string | undefined): string {
  return value ?? "x";
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
      "type/no-unnecessary-condition": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_condition.rs#L5)
