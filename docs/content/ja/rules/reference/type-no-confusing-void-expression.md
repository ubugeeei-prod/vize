---
title: "type/no-confusing-void-expression"
description: "値として使われている void 型の式を検出します。"
---

# `type/no-confusing-void-expression`

値として使われている void 型の式を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

logVoid() は void を返しますが、初期化子がその呼び出しを値として扱っています。

```vue annotate="remove:3"
<script setup lang="ts">
function logVoid(): void {}
const value = logVoid();
</script>
```

## 良い例

logVoid() を文として呼ぶと、void の結果は副作用のためだけに使われます。

```vue annotate="add:3"
<script setup lang="ts">
function logVoid(): void {}
logVoid();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-confusing-void-expression": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_confusing_void_expression.rs#L5)
