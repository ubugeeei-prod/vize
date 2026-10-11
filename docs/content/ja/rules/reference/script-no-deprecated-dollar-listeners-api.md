---
title: "script/no-deprecated-dollar-listeners-api"
---

# `script/no-deprecated-dollar-listeners-api`

Vue 3 で $attrs に統合された $listeners を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`this.$listeners` と `ctx.$listeners` のメンバー参照、および引数に直接渡した `$listeners` が、削除された API を使っています。Vue 3 ではリスナーが属性に統合され、この API は削除されました。

```vue
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

## 良い

参照を `this.$attrs` と setup コンテキストの `ctx.attrs` に置き換え、削除されたリスナー API の使用をなくします。この例は、それぞれの参照元を使えるコンポーネントの文脈を前提にしています。

```vue
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [全ルール](../all.md)
