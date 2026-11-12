---
title: "type/restrict-plus-operands"
description: "加算の両辺が明示的に互換であることを必須にします。"
---

# `type/restrict-plus-operands`

加算の両辺が明示的に互換であることを必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

label は文字列で numbers は配列なので、+ の両辺は互換なプリミティブではありません。

```vue annotate="remove:3,4"
<script setup lang="ts">
const label = "a";
const numbers = [1, 2, 3];
const mixedSum = label + numbers;
</script>
```

## 良い例

label と extra はどちらも文字列なので、+ の両辺は一致します。

```vue annotate="add:3,4"
<script setup lang="ts">
const label = "a";
const extra = "b";
const mixedSum = label + extra;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-plus-operands": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_plus_operands.rs#L5)
