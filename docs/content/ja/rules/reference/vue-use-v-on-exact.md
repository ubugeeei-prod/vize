---
title: "vue/use-v-on-exact"
description: "修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。"
---

# `vue/use-v-on-exact`

修飾子付きのイベント操作と競合するハンドラーに .exact を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## ハンドラーが重なる条件

ネイティブ要素の同じイベントを処理するリスナーを比較します。独自コンポーネントのイベントは、両方に `.native` が付いている場合だけ対象です。イベント名が異なる場合や、キーフィルターに共通するキーがない場合は競合しません。

システム修飾子は、押している必要があるキーを指定します。`@keydown.alt.shift` は Ctrl+Alt+Shift でも動くため、`@keydown.alt.ctrl.shift` と重なる場合は広い方に `.exact` を追加します。`.exact` は指定したシステムキーを許可し、それ以外の Ctrl、Shift、Alt、Meta を拒否します。

複数のキーボード修飾子は選択肢です。`@keydown.up.down` は上・下のどちらでも動き、Ctrl+下では `@keydown.ctrl.down` と重なります。診断範囲は `.exact` が必要なディレクティブのキーで、ハンドラー式は含みません。

## 悪い例

通常の click ハンドラーも Ctrl キーを押しながらのクリックで動くため、別の `.ctrl` ハンドラーと重複します。

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

## 良い例

`.exact` で通常のハンドラーを修飾キーのない click に限定し、Ctrl 専用のハンドラーと分けます。

```vue annotate="add:2,3,4,5,6"
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

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L31)
