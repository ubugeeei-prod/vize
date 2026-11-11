---
title: "type/require-await"
description: "await を含まない async 関数を検出します。"
---

# `type/require-await`

await を含まない async 関数を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

noAwait は async ですが、本体は値を await しません。

```vue annotate="remove:2,3"
<script setup lang="ts">
async function noAwait(): Promise<number> {
  return 1;
}
</script>
```

## 良い例

withAwait は返す前に Promise.resolve(1) を await します。

```vue annotate="add:2,3"
<script setup lang="ts">
async function withAwait(): Promise<number> {
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
      "type/require-await": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/require_await.rs#L5)
