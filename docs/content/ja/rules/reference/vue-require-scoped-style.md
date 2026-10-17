---
title: "vue/require-scoped-style"
---

# `vue/require-scoped-style`

style に scoped を指定する方針を適用します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`.button` の style にスコープがなく、他のコンポーネントの一致する要素にも作用します。

```vue
<style>
.button {
  color: red;
}
</style>
```

## 良い

`scoped` を style に付け、同じセレクタと宣言に Vue のコンポーネントスコープを適用します。

```vue
<style scoped>
.button {
  color: red;
}
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [全ルール](../all.md)
