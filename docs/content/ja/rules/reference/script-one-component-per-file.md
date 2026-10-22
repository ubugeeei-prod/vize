---
title: "script/one-component-per-file"
---

# `script/one-component-per-file`

1 ファイルにコンポーネント定義が複数あることを検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
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
        "script/one-component-per-file": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

ファイルが `Parent` を export し、`components` の中にもう一つの `Child` を定義しています。

```vue
<script lang="ts">
export default {
  name: 'Parent',
  components: {
    Child: { name: 'Child' }
  }
}
</script>
```

## 良い

ファイルが定義するのは `Parent` だけです。

```vue
<script lang="ts">
export default {
  name: 'Parent'
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L30) · [全ルール](../all.md)
