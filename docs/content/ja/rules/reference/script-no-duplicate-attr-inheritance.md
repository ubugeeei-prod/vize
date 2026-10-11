---
title: "script/no-duplicate-attr-inheritance"
---

# `script/no-duplicate-attr-inheritance`

fallthrough 属性を同じコンポーネントで二重に適用する箇所を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
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
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Vue の既定で有効な属性継承を、`inheritAttrs: true` であらためて指定しています。このルールは、ルート要素に `$attrs` を展開するコードがない場合も、この冗長な指定を報告します。

```vue
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

## 良い

`inheritAttrs: false` は継承を無効にする指定であり、空のオプションは既定の継承を暗黙に使います。どちらも冗長な `true` を指定しません。

```vue
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [全ルール](../all.md)
