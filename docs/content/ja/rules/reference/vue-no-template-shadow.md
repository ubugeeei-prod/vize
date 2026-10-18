---
title: "vue/no-template-shadow"
---

# `vue/no-template-shadow`

テンプレート変数が外側の名前を隠す箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

検査はテンプレート内の `v-for`、`v-slot`（`#name` を含む）、従来のスコープ付きスロットの束縛を比較します。外側の束縛を隠す識別子を報告し、同じ要素内では属性が記述された順に比較します。兄弟要素のスコープは独立しています。プロパティのキーや既定値の参照は宣言ではなく、`v-pre` の部分木は検査しません。

script やコンポーネントのプロパティ名は現在の検査範囲に含みません。同じ名前の script 束縛があるだけで、単独のテンプレート束縛を報告することはありません。診断範囲は、エスケープや HTML エンティティを含む元の識別子を示します。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

内側の v-for でも item を宣言し、外側の item を隠しています。

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

## 良い

内側は child に変更し、外側の item と内側の child を分けます。

```vue
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [全ルール](../all.md)
