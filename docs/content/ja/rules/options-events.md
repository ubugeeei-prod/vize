---
title: イベントリスナーのオプション
---
<!-- Reviewed translation; source: rules/options-events.md -->

# イベントリスナーのオプション

## `vue/v-on-event-hyphenation`

既存の `"always"` と `"never"` の文字列指定も使えます。オブジェクト形式の既定値は
`{ style: "always", ignore: [], autofix: false }` です。`ignore` は大文字・小文字を区別する部分文字列です。
`"update:"` は `@update:modelValue` を除外しますが、`"Update:"` や `"update:*"` は除外しません。
重複した値、空白文字だけの値、末尾が `:exit` の値は指定できません。
後から一致する設定はルールのオプション全体を置き換え、`{}` は継承した除外を解除して `autofix` を `false` に戻します。
オプションだけでは無効なルールは有効になりません。`ignoreTags` は対象外です。

`autofix: true` で静的なイベント引数の修正候補を付け、CLI の `--fix` やホストの修正操作で適用します。
`@`／`v-on:`、修飾子、ハンドラ式は維持します。`_` を含む名前は診断だけを残します。
`never` の変換でハイフンを除去するのは、その直後が ASCII の単語文字の場合です。
非 ASCII 文字の前のハイフンは残り、診断が続く場合があります。同じ内容の編集でファイルを書き換えることはありません。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/v-on-event-hyphenation": "warn" },
      ruleOptions: {
        "vue/v-on-event-hyphenation": { style: "always", ignore: ["update:"], autofix: true },
      },
    },
  },
});
```

検査対象のリスナー形式は [Vue ルールのオプション](./options-vue.md#vue-v-on-event-hyphenation)を参照してください。
