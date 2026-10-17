---
title: "vue/valid-v-on"
---

# `vue/valid-v-on`

v-on のイベント名・式・修飾子を検査します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 修飾子の検証

未対応の修飾子は、修飾子の名前ごとに報告します。長い `v-on:click`、動的なイベント引数、オブジェクト形式も対象です。受け付ける名前は `eslint-plugin-vue` 10.9.2 に合わせ、Vue 組み込みのイベント修飾子、完全なキー別名一覧、単一の Unicode コードポイント、有限な十進整数の接頭辞を持つ名前です。整数は JavaScript の `Number.parseInt(name, 10)` と同じように読み取るため、`.13suffix` もこのルールでは有効です。数値キーの非推奨判定と `.native` の移行診断は別のルールが担当します。

ハンドラーなしで動作するのは `.stop` と `.prevent` だけです。`.once`、`.enter`、設定した独自の名前にもハンドラー式が必要です。

独自のイベント処理で使う名前は `linter.ruleOptions` に追加できます。

```json
{
  "linter": {
    "rules": { "vue/valid-v-on": "error" },
    "ruleOptions": {
      "vue/valid-v-on": { "modifiers": ["custom"] }
    }
  }
}
```

これにより `@click.custom="handler"` を使えます。名前は大文字と小文字を区別して完全一致で比較し、組み込みの名前に追加します。ルール自体が無効な場合は有効にしません。範囲別の設定では継承した一覧を置き換え、`[]` でリセットできます。

キーの一覧は、MIT ライセンスの [eslint-plugin-vue のキー別名](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json) を元にしています。出典とライセンスは実装とともに保持しています。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-on": "error"
      },
      "ruleOptions": {
        "vue/valid-v-on": {
          "modifiers": [
            "custom"
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

必要なイベントや式がないか、.sync や .nativeish など未対応の修飾子を使っています。

```vue
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
  <div @click.sync="handleClick"></div>
  <div @click.nativeish="handleClick"></div>
  <div @click.once></div>
</template>
```

## 良い

ハンドラー、式を省略できる動作修飾子、対応するキー修飾子、または引数なしの `v-on` に渡すリスナーのオブジェクトを使います。

```vue
<template>
  <div @click="handleClick"></div>
  <form @submit.prevent></form>
  <input @keyup.page-down="handlePageDown" />
  <input @keyup.あ="handleKey" />
  <div v-on="{ click: handleClick }"></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [全ルール](../all.md)
