---
title: "vapor/prefer-static-class"
description: "文字列リテラルの :class を静的 class に置き換えます。"
---

# `vapor/prefer-static-class`

文字列リテラルの :class を静的 class に置き換えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | あり | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

変化しないクラスの文字列をバインディングで評価しています。

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

## 良い例

同じ panel のクラスを静的な class 属性に指定します。

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vapor ルール](../vapor.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31)
