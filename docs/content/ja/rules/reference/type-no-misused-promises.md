---
title: "type/no-misused-promises"
description: "Promise を扱わない場所に Promise を渡している箇所を検出します。"
---

# `type/no-misused-promises`

Promise を扱わない場所に Promise を渡している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

forEach は void のコールバックを期待しますが、async のコールバックは Promise を返します。

```vue annotate="remove:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach(async (n) => {
  console.log(n);
});
</script>
```

## 良い例

コールバックは async ではないため、forEach が期待する void を返します。

```vue annotate="add:3"
<script setup lang="ts">
const numbers = [1, 2, 3];
numbers.forEach((n) => {
  console.log(n);
});
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-promises": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_promises.rs#L5)
