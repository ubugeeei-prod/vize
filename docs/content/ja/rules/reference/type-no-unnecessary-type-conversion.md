---
title: "type/no-unnecessary-type-conversion"
description: "型も値も変えない変換を検出します。"
---

# `type/no-unnecessary-type-conversion`

型も値も変えない変換を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

label はすでに文字列なので、String(label) は型も値も変えません。

```vue annotate="remove:2,3"
<script setup lang="ts">
const label = "a";
const same = String(label);
</script>
```

## 良い例

count は数値なので、String(count) はそれを文字列へ変換します。

```vue annotate="add:2,3"
<script setup lang="ts">
const count = 1;
const same = String(count);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/no-unnecessary-type-conversion": "warn"
    },
    "typeAware": true
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unnecessary_type_conversion.rs#L5)
