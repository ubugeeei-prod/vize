---
title: Props のルールのオプション
---
<!-- Reviewed translation; source: rules/options-props.md -->

# Props のルールのオプション

## `vue/prop-name-casing`

`{ casing: "snake_case" }` を指定すると、コンポーネント自身が宣言する props 名を
snake_case に統一します。既定は `"camelCase"` です。実行時の配列・オブジェクト、
型リテラル、Options API の宣言を既存のルールで検査します。テンプレート側の属性名は
`vue/attribute-hyphenation` が検査します。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      rules: { "vue/prop-name-casing": "error" },
      ruleOptions: { "vue/prop-name-casing": { casing: "snake_case" } },
    },
  },
});
```

この設定で報告される例:

```vue
<script setup>
defineProps({ displayName: String });
</script>
```

この設定に沿った例:

```vue
<script setup>
defineProps({ display_name: String });
</script>
```

オプションだけではルールは有効になりません。後から一致する範囲別の設定は
オプション全体を置き換え、`{}` で既定の camelCase に戻します。不正な値と
未定義のフィールドは拒否します。upstream の `ignoreProps` は指定できません。

props の名前の変更は公開 API に影響するため、自動修正は行いません。診断の範囲は
ソースに書かれた宣言全体を指します。[eslint-plugin-vue](https://eslint.vuejs.org/rules/prop-name-casing.html)
からの移行では、`["error", "snake_case"]` を上記の `rules` と `ruleOptions` に分けて指定してください。
