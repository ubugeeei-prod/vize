---
title: "vue/no-lone-template"
description: "不要な template 要素を検出します。"
---

# `vue/no-lone-template`

不要な template 要素を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

内側の template には、構造を指定するディレクティブやスロットとしての役割がありません。

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

## 良い例

不要な template を取り除き、div に p を直接入れます。

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32)
