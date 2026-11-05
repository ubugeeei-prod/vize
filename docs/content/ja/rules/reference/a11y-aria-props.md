---
title: "a11y/aria-props"
description: "存在しない ARIA 属性を検出します。"
---

# `a11y/aria-props`

存在しない ARIA 属性を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`aria-lable` は綴りが誤っており、有効な ARIA 属性ではありません。

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

## 良い例

正しい `aria-label` に変更してボタンの名前を指定します。

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L20)
