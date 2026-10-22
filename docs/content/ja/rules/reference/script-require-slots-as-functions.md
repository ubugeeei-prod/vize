---
title: "script/require-slots-as-functions"
---

# `script/require-slots-as-functions`

Vue 3 の $slots プロパティを関数として呼び出します。

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
        "script/require-slots-as-functions": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`render` が `this.$slots.default` を呼び出さずに返しています。これは Vue 2 の vnode 配列です。

```vue
<script lang="ts">
export default {
  render() {
    return this.$slots.default
  }
}
</script>
```

## 良い

`render` は `this.$slots.default()` を呼び出し、その vnode を使います。

```vue
<script lang="ts">
export default {
  render() {
    return this.$slots.default()
  }
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_slots_as_functions.rs#L44) · [全ルール](../all.md)
