---
title: "ecosystem/nuxt-prefer-nuxt-link"
description: "Nuxt の内部リンクに NuxtLink を使います。"
---

# `ecosystem/nuxt-prefer-nuxt-link`

Nuxt の内部リンクに NuxtLink を使います。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

Nuxt の内部ページへの移動に通常の a を使っています。

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

## 良い例

同じ移動先を NuxtLink に指定し、Nuxt のルーターを使います。

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/nuxt-prefer-nuxt-link": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L15)
