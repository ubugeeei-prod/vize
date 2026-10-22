---
title: "script/no-deprecated-data-object-declaration"
---

# `script/no-deprecated-data-object-declaration`

Vue 3 で関数にすべき data オプションのオブジェクト指定を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Options API の `data` がオブジェクトリテラルであり、Vue 3 では受け付けない Vue 2 の形式です。

```vue
<script lang="ts">
export default {
  // `data` must be a function in Vue 3, not an object literal.
  data: {
    count: 0
  }
}
</script>
```

## 良い

`data()` が新しい `{ count: 0 }` を返すようにし、Vue 3 が求める関数形式のデータ宣言にします。

```vue
<script lang="ts">
export default {
  data() {
    return { count: 0 }
  }
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L49) · [全ルール](../all.md)
