---
title: "type/no-unnecessary-type-arguments"
description: "既定値と同じ型引数を検出します。"
---

# `type/no-unnecessary-type-arguments`

既定値と同じ型引数を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

明示した number は、型引数の既定値と同じです。

```vue annotate="remove:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity<number>(1);
</script>
```

## 良い例

identity(1) と呼ぶと、型引数の既定値が使われます。

```vue annotate="add:5"
<script setup lang="ts">
function identity<T = number>(value: T): T {
  return value;
}
const fromIdentity = identity(1);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-arguments": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_arguments.rs#L5)
