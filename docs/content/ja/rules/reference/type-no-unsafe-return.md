---
title: "type/no-unsafe-return"
description: "関数から any 型の値を返している箇所を検出します。"
---

# `type/no-unsafe-return`

関数から any 型の値を返している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

関数は number を返すと宣言していますが、返している anyValue は any です。

```vue annotate="remove:2,4"
<script setup lang="ts">
declare const anyValue: any;
function returnsAny(): number {
  return anyValue;
}
</script>
```

## 良い例

関数は数値 1 を返しており、注釈した戻り値の型と一致します。

```vue annotate="add:3"
<script setup lang="ts">
function returnsAny(): number {
  return 1;
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
      "type/no-unsafe-return": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_return.rs#L5)
