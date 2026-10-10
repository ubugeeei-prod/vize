---
title: "SSR ルール"
---

# SSR ルール

このページに、各ルールの目的・前提・設定・完全な悪い例と良い例をまとめています。変更箇所は行の色で示し、コピーしたコードには完全なソースを保持します。各例に現在の対応範囲も明記しています。


| ルール | 例 | 目的 |
| --- | --- | --- |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [悪い](#ssr-no-browser-globals-in-ssr-bad) · [良い](#ssr-no-browser-globals-in-ssr-good) | SSR で実行されるコードのブラウザー専用グローバル参照を検出します。 |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [悪い](#ssr-no-hydration-mismatch-bad) · [良い](#ssr-no-hydration-mismatch-good) | サーバーとクライアントで一致しないテンプレート値を検出します。 |

[全ルール](./all.md) · [ルール オプション](./options.md) · [ESLint 移行対応表](./migration.md) · [プロジェクトの検査](./cross-file.md) · [子への属性の継承](./project/vue-cross-file-attrs-fallthrough.md)

### `ssr/no-browser-globals-in-ssr`

SSR で実行されるコードのブラウザー専用グローバル参照を検出します。

[悪い例](#ssr-no-browser-globals-in-ssr-bad) · [良い例](#ssr-no-browser-globals-in-ssr-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**悪い**

setup の実行時に `window.innerWidth` を直接読みますが、サーバー上のコンポーネント実行時には `window` が存在しません。

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**良い**

初期の幅をサーバーでも扱える ref の値にし、ブラウザー API の参照を SSR の setup ではなくクライアントで実行する `onMounted` に移します。

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [全ルール](all.md)

### `ssr/no-hydration-mismatch`

サーバーとクライアントで一致しないテンプレート値を検出します。

[悪い例](#ssr-no-hydration-mismatch-bad) · [良い例](#ssr-no-hydration-mismatch-good)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

**設定（Vite+）**

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

<span id="ssr-no-hydration-mismatch-bad"></span>

**悪い**

テンプレートが描画時に `Math.random()` を評価し、同じ段落でもサーバーとクライアントで異なるテキストになり得ます。

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**良い**

新たな乱数ではなく、安定した `seed` の状態を段落に描画します。この Nuxt 形式の例では `useState` が状態を共有し、初期値も定数 `"stable"` です。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [全ルール](all.md)
