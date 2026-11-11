---
title: "type/no-unnecessary-type-assertion"
description: "式の型を変えない型アサーションを検出します。"
---

# `type/no-unnecessary-type-assertion`

式の型を変えない型アサーションを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

known はすでに number なので、number へのアサーションは型を変えません。

```vue annotate="remove:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known as number;
</script>
```

## 良い例

アサーションなしで known を使うと、注釈された number 型のままです。

```vue annotate="add:3"
<script setup lang="ts">
const known: number = 1;
const asserted = known;
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-assertion": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_assertion.rs#L5)
