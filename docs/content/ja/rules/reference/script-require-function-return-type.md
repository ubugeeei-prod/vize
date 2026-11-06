---
title: "script/require-function-return-type"
description: "関数に戻り値の型注釈を指定します。"
---

# `script/require-function-return-type`

関数に戻り値の型注釈を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

`add` と `greet` は引数の型を指定していますが、戻り値の型を省略しています。この明示的な型指定の規則では、戻り値の推論だけでは条件を満たしません。

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
  return a + b
}

function greet(name: string) {
  return `Hello, ${name}`
}
</script>
```

## 良い例

`add` に `: number`、`greet` に `: string` を付け、本体を変えずに戻り値の契約を明示します。

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
  return a + b
}

function greet(name: string): string {
  return `Hello, ${name}`
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
      "script/require-function-return-type": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L55)
