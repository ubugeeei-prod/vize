---
title: "a11y/no-role-presentation-on-focusable"
description: "フォーカス可能な要素への role=\"presentation\" または role=\"none\" の指定を検出します。"
---

# `a11y/no-role-presentation-on-focusable`

フォーカス可能な要素への role="presentation" または role="none" の指定を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

フォーカスできる Billing のリンクに role="presentation" を指定しています。リンク本来の操作可能な役割と矛盾するため、ブラウザーはこの presentation の指定を無視する必要があります。

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## 良い例

矛盾する presentation の指定を除き、Billing への標準のリンクの役割を使います。

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-role-presentation-on-focusable": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19)
