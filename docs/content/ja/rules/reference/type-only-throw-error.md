---
title: "type/only-throw-error"
description: "throw する値が Error 相当であることを必須にします。"
---

# `type/only-throw-error`

throw する値が Error 相当であることを必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

投げている値は文字列であり、Error ではありません。

```vue annotate="remove:3"
<script setup lang="ts">
function fail(): never {
  throw "failure";
}
</script>
```

## 良い例

new Error は同じ失敗の文言に対して Error を作ります。

```vue annotate="add:3"
<script setup lang="ts">
function fail(): never {
  throw new Error("failure");
}
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/only-throw-error": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/only_throw_error.rs#L5)
