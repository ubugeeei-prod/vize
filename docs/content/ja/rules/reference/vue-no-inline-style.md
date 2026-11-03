---
title: "vue/no-inline-style"
description: "インラインの style 属性を検出します。"
---

# `vue/no-inline-style`

インラインの style 属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

静的な style 属性に色の宣言を直接書いています。

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

## 良い例

固定した色にはクラスを使います。ratio に依存する幅の動的な style は、静的属性の検査の対象外です。

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
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
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33)
