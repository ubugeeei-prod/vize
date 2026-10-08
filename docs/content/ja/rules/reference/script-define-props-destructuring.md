---
title: "script/define-props-destructuring"
---

# `script/define-props-destructuring`

defineProps の分割代入スタイルを指定した方針に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JS / TS script。Options API または script setup の対象は例を参照してください。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`defineProps` の結果を分割代入せず、単一の `props` 変数に代入しており、既定の分割代入の推奨に従っていません。

```vue
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

## 良い

オブジェクトパターンで `foo` と `bar` を直接取り出し、省略可能な `bar` に既定値を付けます。Vue 3.5 以降のリアクティブな props 分割代入を前提とし、設定を `never` にした場合は逆の形式を推奨します。

```vue
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [全ルール](../all.md)
