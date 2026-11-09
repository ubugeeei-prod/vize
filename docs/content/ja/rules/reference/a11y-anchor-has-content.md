---
title: "a11y/anchor-has-content"
description: "リンクに支援技術で読める内容を用意します。"
---

# `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

描画結果がリンクのアクセシブルな内容を必ず提供するコンポーネント名を指定します。PascalCase と kebab-case は同じ名前として扱います。オプションだけではルールを有効にしません。後の設定レイヤーは一覧全体を置き換え、[] で継承した名前を消します。

## 悪い例

`/app` へのリンクが空で、移動先を説明する内容がありません。

```vue annotate="remove:2"
<template>
<a href="/app"></a>
</template>
```

## 良い例

標準の画像の `alt` テキストが、同じ移動先へのリンクにアクセシブルな内容を提供します。

```vue annotate="add:2"
<template>
<a href="/app"><img src="/badge.png" alt="Get the app" /></a>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/anchor-has-content": "warn"
    },
    "ruleOptions": {
      "a11y/anchor-has-content": {
        "accessibleChildren": [
          "RetryImage"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L24)
