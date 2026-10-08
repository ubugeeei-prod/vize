---
title: "a11y/no-access-key"
---

# `a11y/no-access-key`

環境のショートカットと衝突し得る accesskey を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`accesskey="s"` がブラウザーや支援技術のショートカットと競合する可能性があります。

```vue
<template>
  <button accesskey="s">Save</button>
</template>
```

## 良い

accesskey を取り除き、通常の Save ボタンは残します。

```vue
<template>
  <button>Save</button>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [全ルール](../all.md)
