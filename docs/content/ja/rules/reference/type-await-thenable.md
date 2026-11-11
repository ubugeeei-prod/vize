---
title: "type/await-thenable"
description: "thenable でない値を await している箇所を検出します。"
---

# `type/await-thenable`

thenable でない値を await している箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

plainNumber は数値を返すため、thenable でない値を await しています。

```vue annotate="remove:2,3,4,6"
<script setup lang="ts">
function plainNumber(): number {
  return 1;
}
async function run(): Promise<number> {
  return await plainNumber();
}
</script>
```

## 良い例

Promise.resolve(1) は thenable なので、実際の Promise を待っています。

```vue annotate="add:3"
<script setup lang="ts">
async function run(): Promise<number> {
  return await Promise.resolve(1);
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
      "type/await-thenable": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/await_thenable.rs#L5)
