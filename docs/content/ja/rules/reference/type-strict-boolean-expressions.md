---
title: "type/strict-boolean-expressions"
description: "script とテンプレートの条件式で安全な真偽判定を使います。"
---

# `type/strict-boolean-expressions`

script とテンプレートの条件式で安全な真偽判定を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC の script とテンプレートの型情報。例に示した構文が対象です。

typeAware とこのルールを明示的に有効にします。既定では null または undefined を含み得る数値は許可されず、どちらも含まない数値は許可されます。

このルールは TypeScript プロジェクト（`tsconfig.json`）の型情報を使うため、設定で `typeAware: true` を指定しています。`typeAware` だけでは、個別に有効化するルールは有効になりません。

## 悪い例

`if (count)` は、undefined を含み得る数値をそのまま真偽判定に使い、明示的な真偽値の比較をしていません。数値のゼロと値の未指定を、どちらも偽として扱ってしまいます。

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

## 良い例

`count !== undefined && count > 0` で、値が undefined でないことと正の数であることを別々に検査します。省略可能な値を絞り込んでから、真偽値を返す条件式で判定します。

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/strict-boolean-expressions": "warn"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7)
