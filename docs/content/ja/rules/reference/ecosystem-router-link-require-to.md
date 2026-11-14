---
title: "ecosystem/router-link-require-to"
description: "RouterLink / NuxtLink に to を指定します。"
---

# `ecosystem/router-link-require-to`

RouterLink / NuxtLink に to を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `ecosystem` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

SFC の単一ルート要素がリンクである場合、親から属性を継承できるため対象外になることがあります。この例は別の要素の内側にあるリンクで、明示的な遷移先が必要です。

## 悪い例

nav の子要素である RouterLink に to がなく、SFC のルート要素を通じた属性の継承も利用できません。

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## 良い例

nav の内側にあるリンクに `to="/settings"` で移動先を明示します。

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/router-link-require-to": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L15)
