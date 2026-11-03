---
title: "ecosystem/vue-i18n-no-missing-key"
description: "SFC 内の翻訳データに存在しない静的キーを検出します。"
---

# `ecosystem/vue-i18n-no-missing-key`

SFC 内の翻訳データに存在しない静的キーを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `ecosystem` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートは auth.missing を参照しますが、ローカルの英語メッセージには auth.login しかありません。

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## 良い例

ローカルのメッセージにある auth.login を参照します。

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17)
