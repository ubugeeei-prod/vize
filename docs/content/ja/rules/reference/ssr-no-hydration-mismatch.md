---
title: "ssr/no-hydration-mismatch"
---

# `ssr/no-hydration-mismatch`

サーバーとクライアントで一致しないテンプレート値を検出します。

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
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

テンプレートが描画時に `Math.random()` を評価し、同じ段落でもサーバーとクライアントで異なるテキストになり得ます。

```vue
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## 良い

新たな乱数ではなく、安定した `seed` の状態を段落に描画します。この Nuxt 形式の例では `useState` が状態を共有し、初期値も定数 `"stable"` です。

```vue
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [全ルール](../all.md)
