---
title: "type/no-duplicate-type-constituents"
description: "共用体や交差型で重複する構成要素を検出します。"
---

# `type/no-duplicate-type-constituents`

共用体や交差型で重複する構成要素を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

同じ共用体に string が二度現れるため、二つ目の構成要素は何も加えません。

```vue annotate="remove:2"
<script setup lang="ts">
type Dup = string | string;
</script>
```

## 良い例

別名は string を一度だけ残します。

```vue annotate="add:2"
<script setup lang="ts">
type Dup = string;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-duplicate-type-constituents": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_duplicate_type_constituents.rs#L5)
