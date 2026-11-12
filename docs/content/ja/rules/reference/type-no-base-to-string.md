---
title: "type/no-base-to-string"
description: "Object.prototype.toString() に落ちる値の文字列化を検出します。"
---

# `type/no-base-to-string`

Object.prototype.toString() に落ちる値の文字列化を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

plain は普通のオブジェクトなので、toString() は Object.prototype.toString() に落ちます。

```vue annotate="remove:2,3"
<script setup lang="ts">
const plain = { a: 1 };
plain.toString();
</script>
```

## 良い例

label はすでに文字列なので、toString() は文字列の変換を使います。

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
label.toString();
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-base-to-string": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_base_to_string.rs#L5)
