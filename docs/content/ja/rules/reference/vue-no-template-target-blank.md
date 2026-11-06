---
title: "vue/no-template-target-blank"
description: "target=_blank の外部リンクに適切な rel を指定します。"
---

# `vue/no-template-target-blank`

target=_blank の外部リンクに適切な rel を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

外部リンクを target=_blank で開きますが、必要な rel の保護がありません。

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

## 良い例

同じリンクに noopener noreferrer を指定します。

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-target-blank": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33)
