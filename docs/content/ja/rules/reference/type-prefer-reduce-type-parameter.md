---
title: "type/prefer-reduce-type-parameter"
description: "Array#reduce の初期値アサーションより型引数を推奨します。"
---

# `type/prefer-reduce-type-parameter`

Array#reduce の初期値アサーションより型引数を推奨します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

初期値を number[] とアサーションしており、reduce の型引数を使っていません。

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce((all, n) => [...all, n], [] as number[]);
</script>
```

## 良い例

reduce<number[]> は、初期配列をアサーションせずに累積値の型を示します。

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const collected = numbers.reduce<number[]>((all, n) => [...all, n], []);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/prefer-reduce-type-parameter": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/prefer_reduce_type_parameter.rs#L5)
