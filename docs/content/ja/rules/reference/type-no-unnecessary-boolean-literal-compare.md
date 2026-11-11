---
title: "type/no-unnecessary-boolean-literal-compare"
description: "真偽値リテラルとの不要な比較を検出します。"
---

# `type/no-unnecessary-boolean-literal-compare`

真偽値リテラルとの不要な比較を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

flag はすでに真偽値なので、true との比較は検査を加えません。

```vue annotate="remove:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag === true;
}
</script>
```

## 良い例

flag を返すと、真偽値をそのまま使います。

```vue annotate="add:3"
<script setup lang="ts">
function isFlag(flag: boolean): boolean {
  return flag;
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
      "type/no-unnecessary-boolean-literal-compare": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_boolean_literal_compare.rs#L5)
