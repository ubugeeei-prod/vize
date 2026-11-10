---
title: "a11y/interactive-supports-focus"
description: "操作可能な role の要素をフォーカス可能にします。"
---

# `a11y/interactive-supports-focus`

操作可能な role の要素をフォーカス可能にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

eslint-plugin-vuejs-accessibility と異なり、フォーカスできる option を持つ listbox（roving tabindex）や、combobox が `aria-activedescendant` で操作する listbox は自身のタブ位置を必要としません。条件分岐のすべてが整数になる `tabindex` の束縛（`:tabindex="active ? 0 : -1"`）は tabindex として扱い、`:aria-hidden="false"` は要素を隠しません。

## 悪い例

span に button のロールとクリックハンドラーを付けても、キーボードでフォーカスできる要素にはなりません。

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## 良い例

フォーカスできる標準の button に変更し、同じ `open` を実行します。

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/interactive-supports-focus": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L46)
