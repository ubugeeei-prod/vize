---
title: 型ルール オプション
---

<!-- Reviewed translation; source: rules/options-type.md; scope: introduction and rule options prose -->

# 型ルール オプション

条件式で、数値の 0・空文字列・存在しないオブジェクトをどう扱うか明示したいときに使う設定です。
`type/strict-boolean-expressions` は自分で有効にする必要があり、型情報を使う lint も必要です。
オプションを設定するだけではルールは有効になりません。

下の設定と改善前・改善後の例から始め、`vp run lint` を実行して指摘された条件式を確認してください。
重要度やファイルごとの設定の置き換えは [ルールオプションのガイド](./options.md) を参照してください。

## `type/strict-boolean-expressions`

Vite+ では、次のように `lint.vize.typeAware` と `lint.vize.rules` を設定してください。
このルールは、どのプリセットにも含まれていません。
スタンドアロン設定では `linter.rules` と `linter.ruleOptions` に同じ設定を置きます。
ネイティブの型チェック用コードは、すでに厳密な型チェックを使っています。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      typeAware: true,
      rules: {
        "type/strict-boolean-expressions": "error",
      },
      ruleOptions: {
        "type/strict-boolean-expressions": {
          allowString: false,
          allowNumber: false,
          allowNullableObject: false,
        },
      },
    },
  },
});
```

悪い例: この設定では、次の条件式が指摘されます。

```vue
<script setup lang="ts">
defineProps<{ count: number; title: string; element?: HTMLElement }>();
</script>
<template>
  <p v-if="count">Items</p>
  <p v-show="title">Title</p>
  <p v-if="element">Element</p>
</template>
```

良い例: 0・空文字列・存在しないオブジェクトの扱いを、比較演算で明示します。

```vue
<template>
  <p v-if="count > 0">Items</p>
  <p v-show="title !== ''">Title</p>
  <p v-if="element != null">Element</p>
</template>
```

デフォルトでは `allowString`・`allowNumber`・`allowNullableObject` が `true` です。
`allowNullableBoolean`・`allowNullableString`・`allowNullableNumber`・`allowNullableEnum`・`allowAny` は
`false` です。どの項目も真偽値を取ります。

オプションだけではルールは有効になりません。後から一致した設定は option object 全体を置き換えます。
明示的な `off` が優先されます。アサーション関数、配列の述語関数、外部テンプレートと Pug はこのルールの対象外です。
