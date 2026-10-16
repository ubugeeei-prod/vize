---
title: "vue/use-v-on-exact"
---

# `vue/use-v-on-exact`

修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## ハンドラーが重なる条件

ネイティブ要素の同じイベントを処理するリスナーを比較します。独自コンポーネントのイベントは、両方に `.native` が付いている場合だけ対象です。イベント名が異なる場合や、キーフィルターに共通するキーがない場合は競合しません。

システム修飾子は、押している必要があるキーを指定します。`@keydown.alt.shift` は Ctrl+Alt+Shift でも動くため、`@keydown.alt.ctrl.shift` と重なる場合は広い方に `.exact` を追加します。`.exact` は指定したシステムキーを許可し、それ以外の Ctrl、Shift、Alt、Meta を拒否します。

複数のキーボード修飾子は選択肢です。`@keydown.up.down` は上・下のどちらでも動き、Ctrl+下では `@keydown.ctrl.down` と重なります。診断範囲は `.exact` が必要なディレクティブのキーで、ハンドラー式は含みません。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

通常の click ハンドラーも Ctrl キーを押しながらのクリックで動くため、別の `.ctrl` ハンドラーと重複します。

```vue
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## 良い

`.exact` で通常のハンドラーを修飾キーのない click に限定し、Ctrl 専用のハンドラーと分けます。

```vue
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31) · [全ルール](../all.md)
