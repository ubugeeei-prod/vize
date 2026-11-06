---
title: "vapor/require-vapor-attribute"
description: "Vapor 向けの script setup に vapor 属性を付ける規約を示します。"
---

# `vapor/require-vapor-attribute`

Vapor 向けの script setup に vapor 属性を付ける規約を示します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | SFC lint では未対応 | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

> [!WARNING]
> 現在の対応：`no-sfc-finding`。 このルール ID は設定できますが、現在の SFC lint はまだ診断を出しません。良い例は意図する書き方を示します。

このルールは、検査用のコールバックが空の仮実装です。vapor 属性は Vapor でのコンパイルを選択するものですが、現在の linter は属性がないことをこの ID では検出しません。

## 悪い例

script setup に Vapor の指定がありません。これは規約を示す例で、現在の空のコールバックは診断を生成しません。

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## 良い例

vapor を追加して Vapor のコンパイルを選択します。修正方針を示す例であり、現在のリンターがこのルールの違反を検出するという意味ではありません。

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="設定できる-id-現在の-sfc-検出なし"></span>

## 設定できる ID（現在の SFC lint では診断されません）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vapor/require-vapor-attribute": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vapor ルール](../vapor.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17)
