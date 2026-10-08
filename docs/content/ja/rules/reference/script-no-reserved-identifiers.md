---
title: "script/no-reserved-identifiers"
---

# `script/no-reserved-identifiers`

Vue コンパイラーが予約した識別子の宣言を検出します。

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
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

変数名 `__props`、`__emit`、`__sfc__` が Vue コンパイラーの生成コード用に予約された識別子と重なっています。

```vue
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

## 良い

通常の名前 `props`、`emit`、`componentData` を使い、props と emits の宣言を保ったまま生成用識別子との重複を避けます。

```vue
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [全ルール](../all.md)
