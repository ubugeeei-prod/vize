---
title: "type/return-await"
description: "try の中で Promise を返すときは `return await` を使います。"
---

# `type/return-await`

try の中で Promise を返すときは `return await` を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

try の中で await せずに getNumber() を返すと、その catch で拒否を受け取れません。

```vue annotate="remove:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return getNumber();
  } catch {
    return 0;
  }
}
</script>
```

## 良い例

return await getNumber() にすると、拒否は同じ try と catch の中に残ります。

```vue annotate="add:7"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
async function returnsInTry(): Promise<number> {
  try {
    return await getNumber();
  } catch {
    return 0;
  }
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
      "type/return-await": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/return_await.rs#L5)
