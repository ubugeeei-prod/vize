---
title: "vue/no-deprecated-functional-template"
description: "SFC の template で削除済みの functional 属性を検出します。"
---

# `vue/no-deprecated-functional-template`

SFC の template で削除済みの functional 属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

SFC の template に廃止された functional を指定し、旧来の props 経由の参照を使っています。

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

## 良い例

functional を取り除き、コンポーネントの msg を直接参照します。

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
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
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57)
