---
title: "ecosystem/void-link-require-href"
---

# `ecosystem/void-link-require-href`

Void Vue の Link に href を指定します。

既定の重大度: `error`  
プリセット: `ecosystem`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

## 良い

```vue
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [全ルール](../all.md)
