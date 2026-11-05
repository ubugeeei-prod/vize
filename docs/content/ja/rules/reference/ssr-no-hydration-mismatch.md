---
title: "ssr/no-hydration-mismatch"
description: "サーバーとクライアントで一致しないテンプレート値を検出します。"
---

# `ssr/no-hydration-mismatch`

サーバーとクライアントで一致しないテンプレート値を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

テンプレートが描画時に `Math.random()` を評価し、同じ段落でもサーバーとクライアントで異なるテキストになり得ます。

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

## 良い例

新たな乱数ではなく、安定した `seed` の状態を段落に描画します。この Nuxt 形式の例では `useState` が状態を共有し、初期値も定数 `"stable"` です。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [SSR ルール](../ssr.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122)
