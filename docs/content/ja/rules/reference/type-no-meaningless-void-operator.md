---
title: "type/no-meaningless-void-operator"
description: "すでに void 相当の式に付いた void 演算子を検出します。"
---

# `type/no-meaningless-void-operator`

すでに void 相当の式に付いた void 演算子を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

logVoid() はすでに void なので、void 演算子は結果を変えません。

```vue annotate="remove:2,3"
<script setup lang="ts">
function logVoid(): void {}
void logVoid();
</script>
```

## 良い例

getNumber() は Promise を返すため、void はその thenable を意図して捨てます。

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
function getNumber(): Promise<number> {
  return Promise.resolve(1);
}
void getNumber();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-meaningless-void-operator": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_meaningless_void_operator.rs#L5)
