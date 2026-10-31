---
title: イベントリスナーのオプション
---
<!-- Reviewed translation; source: rules/options-events.md -->

# イベントリスナーのオプション

## `vue/v-on-event-hyphenation`

既存の `"always"` と `"never"` の文字列指定も使えます。オブジェクト形式の既定値は
`{ style: "always", ignore: [] }` です。`ignore` は大文字・小文字を区別する部分文字列です。
`"update:"` は `@update:modelValue` を除外しますが、`"Update:"` や `"update:*"` は除外しません。
重複した値、空白文字だけの値、末尾が `:exit` の値は指定できません。
後から一致する設定はルールのオプション全体を置き換え、`{}` は継承した除外を解除します。
オプションだけでは無効なルールは有効になりません。自動修正と `ignoreTags` は対象外です。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/v-on-event-hyphenation": "warn" },
      ruleOptions: {
        "vue/v-on-event-hyphenation": { style: "always", ignore: ["update:"] },
      },
    },
  },
});
```

検査対象のリスナー形式は [Vue ルールのオプション](./options-vue.md#vue-v-on-event-hyphenation)を参照してください。
