---
title: "type/no-deprecated"
description: "非推奨の宣言を使っている箇所を検出します。"
---

# `type/no-deprecated`

非推奨の宣言を使っている箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

legacy には @deprecated が付いており、呼び出しがその宣言を使っています。

```vue annotate="remove:2,3,6"
<script setup lang="ts">
/** @deprecated use current() */
function legacy(): number {
  return 1;
}
const legacyValue = legacy();
</script>
```

## 良い例

current には非推奨の印がないため、呼び出しはサポートされた宣言を使います。

```vue annotate="add:2,5"
<script setup lang="ts">
function current(): number {
  return 1;
}
const currentValue = current();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-deprecated": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_deprecated.rs#L5)
