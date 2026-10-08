---
title: "script/require-typed-object-prop"
---

# `script/require-typed-object-prop`

Object / Array の prop に具体的な型を指定します。

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
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

裸の `Object` と `Array` は広い実行時の分類しか表さず、`user` や `items` の要素の形を明示する静的な型がありません。

```vue
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

## 良い

`PropType<User>` と `PropType<User[]>` を付け、実行時のコンストラクターを保ったままオブジェクトと要素の型を指定します。

```vue
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [全ルール](../all.md)
