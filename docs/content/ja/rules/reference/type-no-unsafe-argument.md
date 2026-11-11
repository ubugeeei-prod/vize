---
title: "type/no-unsafe-argument"
description: "`any` 型の値を関数引数に渡している箇所を検出します。"
---

# `type/no-unsafe-argument`

`any` 型の値を関数引数に渡している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

anyValue は any であり、number を期待する引数へ渡されています。

```vue annotate="remove:2,6"
<script setup lang="ts">
declare const anyValue: any;
function takesNumber(value: number): number {
  return value;
}
takesNumber(anyValue);
</script>
```

## 良い例

呼び出しは数値 1 を number の引数へ渡します。

```vue annotate="add:5"
<script setup lang="ts">
function takesNumber(value: number): number {
  return value;
}
takesNumber(1);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unsafe-argument": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_argument.rs#L5)
