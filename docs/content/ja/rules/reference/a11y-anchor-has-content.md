---
title: "a11y/anchor-has-content"
description: "リンクに支援技術で読める内容を用意します。"
---

# `a11y/anchor-has-content`

リンクに支援技術で読める内容を用意します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`/settings` へのリンクが空で、移動先を説明する内容がありません。

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

## 良い例

同じリンクに `Settings` の文字を入れ、移動先を示します。

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
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
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [アクセシビリティのルール](../accessibility.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L20)
