---
title: "script/valid-define-emits"
---

# `script/valid-define-emits`

defineEmits の重複や型と実行時引数の併用を検出します。

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
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

同じ `defineEmits` に型引数と実行時配列 `["save"]` の両方を渡し、併用できない二つの宣言形式を混ぜています。

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

## 良い

実行時の引数を除き、`save` の宣言を型ベースの一つの形式に統一します。

```vue
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [全ルール](../all.md)
