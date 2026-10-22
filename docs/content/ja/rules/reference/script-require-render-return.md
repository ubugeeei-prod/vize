---
title: "script/require-render-return"
---

# `script/require-render-return`

render 関数が値を返すことを必須にします。

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
        "script/require-render-return": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`render` 関数が分岐の中で `this.count` を読むだけで、値を返していません。

```vue
<script lang="ts">
export default {
  render() {
    if (this.ready) {
      this.count
    }
  }
}
</script>
```

## 良い

`render` 関数は `this.count` を返します。

```vue
<script lang="ts">
export default {
  render() {
    return this.count
  }
}
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_render_return.rs#L35) · [全ルール](../all.md)
