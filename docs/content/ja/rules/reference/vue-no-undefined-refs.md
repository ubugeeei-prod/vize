---
title: "vue/no-undefined-refs"
description: "テンプレート内の未定義変数参照を検出します。"
---

# `vue/no-undefined-refs`

テンプレート内の未定義変数参照を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

`globals` には、`unplugin-auto-import` の自動インポートなど、ビルド時にすべてのテンプレートで使えるようになる名前を指定します。Nuxt プロジェクトでは、生成済みの `.nuxt/types/imports.d.ts` から自動インポートの名前を `vize lint` が読み取って加えるため、指定は不要です。`nuxi prepare` がこのファイルを生成するまでは、通知を表示してこのルールを省略します。名前は完全一致で照合します。オプションだけではルールは有効になりません。後の設定は一覧全体を置き換え、空の一覧は継承した名前を消します。

## 悪い例

script は message しか宣言していませんが、テンプレートが missing を参照しています。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## 良い例

script setup で宣言済みの message を補間で参照し、未定義の名前を取り除きます。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-undefined-refs": "warn"
    },
    "ruleOptions": {
      "vue/no-undefined-refs": {
        "globals": [
          "formatCount"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L13)
