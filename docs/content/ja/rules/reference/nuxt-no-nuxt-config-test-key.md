---
title: "nuxt/no-nuxt-config-test-key"
---

# `nuxt/no-nuxt-config-test-key`

Nuxt が自動判定する test 環境の手動設定を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Nuxt 設定ファイル（nuxt.config.ts）  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

エクスポートした Nuxt 設定で `test` キーに真偽値 `true` を指定しており、このルールが拒否する旧形式の設定です。

`nuxt.config.ts`

```ts
export default defineNuxtConfig({ test: true });
```

## 良い

空の設定にすることで、真偽値の `test` プロパティを取り除きます。テスト設定のオブジェクトまで禁止する例ではありません。

`nuxt.config.ts`

```ts
export default defineNuxtConfig({});
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [全ルール](../all.md)
