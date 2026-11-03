---
title: "vue/no-useless-template-attributes"
description: "template 要素の効果がない属性を検出します。"
---

# `vue/no-useless-template-attributes`

template 要素の効果がない属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

条件付きの template に class を指定していますが、この構造用のラッパーは DOM 要素を描画しません。

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

## 良い例

実際に表示する p に class を移し、構造を指定する template の v-if は残します。

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
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
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32)
