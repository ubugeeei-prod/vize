---
title: "type/no-unsafe-unary-minus"
description: "number でも bigint でもない値の単項マイナスを検出します。"
---

# `type/no-unsafe-unary-minus`

number でも bigint でもない値の単項マイナスを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

label は文字列なので、単項マイナスは number でも bigint でもない値に付いています。

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const negated = -label;
</script>
```

## 良い例

count は数値なので、単項マイナスはその数値を否定します。

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const negated = -count;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-unary-minus": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_unary_minus.rs#L5)
