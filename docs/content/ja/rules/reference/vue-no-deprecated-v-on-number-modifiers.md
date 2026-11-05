---
title: "vue/no-deprecated-v-on-number-modifiers"
description: "v-on の削除済みの数値 keyCode 修飾子を検出します。"
---

# `vue/no-deprecated-v-on-number-modifiers`

v-on の削除済みの数値 keyCode 修飾子を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 数値修飾子の選択と修正

最初に十進整数として読み取れる修飾子だけを判定します。`@keyup.13.27="submit"` では
`13` を報告し、`@keyup.enter.27="submit"` に修正します。次の修正で `27` を `escape` にできます。
`@keyup.13suffix="submit"` も `@keyup.enter="submit"` に修正します。0〜9 は対象外のため、
`@keyup.0.13="submit"` はこのルールでは報告しません。`999` など未知のコードは報告だけ行います。
Vue 2 と petite-vue では従来どおり報告しません。

## 悪い例

キーを、廃止された数値コード 13 と 27 で指定しています。

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## 良い例

キーの名前を使い、enter と esc の修飾子に変更します。

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
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
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L45)
