---
title: "script/valid-define-props"
---

# `script/valid-define-props`

defineProps の重複や型と実行時引数の併用を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `error`  
プリセット: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

同じ `defineProps` に型引数 `{ title: string }` と実行時引数 `{ title: String }` の両方を渡しています。コンパイラーはこの併用を許可しません。

```vue
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

## 良い

実行時オブジェクトを除き、二つの形式を併用せず `title` の型ベースの宣言だけを残します。

```vue
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [全ルール](../all.md)
