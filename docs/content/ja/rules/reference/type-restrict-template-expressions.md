---
title: "type/restrict-template-expressions"
description: "テンプレートリテラルの式を文字列と互換にすることを必須にします。"
---

# `type/restrict-template-expressions`

テンプレートリテラルの式を文字列と互換にすることを必須にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

テンプレートは配列を補間しており、文字列と互換な式ではありません。

```vue annotate="remove:2,3"
<script setup lang="ts">
const numbers = [1, 2, 3];
const listText = `${numbers}`;
</script>
```

## 良い例

テンプレートは文字列である label を補間しています。

```vue annotate="add:2,3"
<script setup lang="ts">
const label = "a";
const listText = `${label}`;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/restrict-template-expressions": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/restrict_template_expressions.rs#L5)
