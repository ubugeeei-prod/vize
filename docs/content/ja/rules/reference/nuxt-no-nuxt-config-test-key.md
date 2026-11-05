---
title: "nuxt/no-nuxt-config-test-key"
description: "Nuxt 設定の test キーに旧形式の真偽値を指定する箇所を検出します。"
---

# `nuxt/no-nuxt-config-test-key`

Nuxt 設定の test キーに旧形式の真偽値を指定する箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `nuxt` | なし | なし |

適用範囲：Nuxt 設定ファイル（nuxt.config.ts）。

## 悪い例

エクスポートした Nuxt 設定で `test` キーに真偽値 `true` を指定しており、このルールが拒否する旧形式の設定です。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

## 良い例

空の設定にすることで、真偽値の `test` プロパティを取り除きます。テスト設定のオブジェクトまで禁止する例ではありません。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [エコシステムと Nuxt のルール](../ecosystem.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16)
