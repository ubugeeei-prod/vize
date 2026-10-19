---
title: "script/component-options-name-casing"
---

# `script/component-options-name-casing`

コンポーネントの name オプションを PascalCase に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

コンポーネントの `name: 'my-component'` が kebab-case です。このルールは文字列リテラルのコンポーネント名に PascalCase を求めます。

```vue
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

## 良い

`MyComponent` は大文字で始まり、英数字だけで構成されるため、名前の検査条件を満たします。

```vue
<script lang="ts">
export default {
  name: 'MyComponent'
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [全ルール](../all.md)
