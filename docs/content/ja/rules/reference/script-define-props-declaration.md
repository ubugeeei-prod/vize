---
title: "script/define-props-declaration"
---

# `script/define-props-declaration`

defineProps を型による宣言形式に揃えます。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
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
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`defineProps({ title: String })` は実行時オブジェクトを渡しており、このルールが推奨する型ベースの props 宣言と異なります。

```vue
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

## 良い

`defineProps<{ title: string }>()` の型引数で `title` を宣言し、実行時の宣言引数を使わずに `props.title` の参照を保ちます。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [全ルール](../all.md)
