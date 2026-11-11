---
title: "type/no-redundant-type-constituents"
description: "効果がない共用体や交差型のメンバーを検出します。"
---

# `type/no-redundant-type-constituents`

効果がない共用体や交差型のメンバーを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

unknown は string をすでに含むため、string は共用体を狭めません。

```vue annotate="remove:2"
<script setup lang="ts">
type Redundant = unknown | string;
</script>
```

## 良い例

string と number は、どちらも共用体に意味のある構成要素です。

```vue annotate="add:2"
<script setup lang="ts">
type Redundant = string | number;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-redundant-type-constituents": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_redundant_type_constituents.rs#L5)
