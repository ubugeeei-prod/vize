---
title: "vue/no-multi-spaces"
description: "連続する不要な空白を検出します。"
---

# `vue/no-multi-spaces`

連続する不要な空白を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

属性同士や、要素名と最初の属性の間に空白が二つあります。

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

## 良い例

同じ class と id 属性を保ち、属性間の連続した空白を一つの空白に揃えます。

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multi-spaces": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26)
