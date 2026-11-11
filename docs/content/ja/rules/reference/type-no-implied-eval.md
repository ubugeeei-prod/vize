---
title: "type/no-implied-eval"
description: "文字列を渡す動的なコード実行を検出します。"
---

# `type/no-implied-eval`

文字列を渡す動的なコード実行を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

setTimeout に文字列を渡しており、その文字列が動的なコードとして実行されます。

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
setTimeout("logVoid()", 10);
</script>
```

## 良い例

setTimeout に関数を渡しているため、文字列は評価されません。

```vue annotate="add:2"
<script setup lang="ts">
setTimeout(() => {}, 10);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-implied-eval": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_implied_eval.rs#L5)
