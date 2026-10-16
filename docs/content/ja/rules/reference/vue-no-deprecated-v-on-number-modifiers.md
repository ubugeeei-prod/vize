---
title: "vue/no-deprecated-v-on-number-modifiers"
---

# `vue/no-deprecated-v-on-number-modifiers`

v-on の削除済みの数値 keyCode 修飾子を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
自動修正: 対応するキー名がある数値を置換します。未知のコードは自動修正しません\
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 数値修飾子の選択と修正

最初に十進整数として読み取れる修飾子だけを判定します。`@keyup.13.27="submit"` では
`13` を報告し、`@keyup.enter.27="submit"` に修正します。次の修正で `27` を `escape` にできます。
`@keyup.13suffix="submit"` も `@keyup.enter="submit"` に修正します。0〜9 は対象外のため、
`@keyup.0.13="submit"` はこのルールでは報告しません。`999` など未知のコードは報告だけ行います。
Vue 2 と petite-vue では従来どおり報告しません。

## 悪い

キーを、廃止された数値コード 13 と 27 で指定しています。

```vue
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## 良い

キーの名前を使い、enter と esc の修飾子に変更します。

```vue
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [全ルール](../all.md)
