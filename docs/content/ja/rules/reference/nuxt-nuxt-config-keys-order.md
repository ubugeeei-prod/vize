---
title: "nuxt/nuxt-config-keys-order"
description: "Nuxt 設定のプロパティを推奨順に並べます。"
---

# `nuxt/nuxt-config-keys-order`

Nuxt 設定のプロパティを推奨順に並べます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `nuxt` | あり | なし |

適用範囲：Nuxt 設定ファイル（nuxt.config.ts）。

## 悪い例

設定内で `ssr` が `modules` より先に置かれ、このルールが推奨する Nuxt 設定キーの順序が逆になっています。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

## 良い例

`modules` を `ssr` より前に移し、各値を保ったまま指定の順序にそろえます。変更するのは配置であり、設定値の意味ではありません。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24)
