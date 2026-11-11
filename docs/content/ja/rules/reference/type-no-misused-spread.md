---
title: "type/no-misused-spread"
description: "互換性のない位置へのスプレッドを検出します。"
---

# `type/no-misused-spread`

互換性のない位置へのスプレッドを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

文字列を配列へスプレッドすると、互換性のない位置に値を置きます。

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const chars = [...label];
</script>
```

## 良い例

配列を配列へスプレッドすると、要素の型は互換のままです。

```vue annotate="add:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const chars = [...numbers];
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-misused-spread": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_misused_spread.rs#L5)
