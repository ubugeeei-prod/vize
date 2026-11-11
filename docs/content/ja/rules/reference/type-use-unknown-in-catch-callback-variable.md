---
title: "type/use-unknown-in-catch-callback-variable"
description: "Promise の catch コールバック引数に unknown 注釈を必須にします。"
---

# `type/use-unknown-in-catch-callback-variable`

Promise の catch コールバック引数に unknown 注釈を必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

catch のコールバックは error を明示的な unknown 注釈なしで受け取っています。

```vue annotate="remove:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error) => {
  console.log(error);
});
</script>
```

## 良い例

catch のコールバックは error を unknown と注釈しています。

```vue annotate="add:5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
getNumber().catch((error: unknown) => {
  console.log(error);
});
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/use-unknown-in-catch-callback-variable": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/use_unknown_in_catch_callback_variable.rs#L5)
