---
title: "vue/no-constant-condition"
description: "テンプレート式の定数条件を検出します。"
---

# `vue/no-constant-condition`

テンプレート式の定数条件を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

三項演算子の条件が常に真です。

```vue annotate="remove:2"
<template>
  <p>{{ true ? primary : fallback }}</p>
</template>
```

## 良い例

リアクティブな `ready` の値で分岐します。

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-constant-condition": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L82)
