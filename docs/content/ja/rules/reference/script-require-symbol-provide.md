---
title: "script/require-symbol-provide"
---

# `script/require-symbol-provide`

provide / inject のキーに衝突しにくい Symbol を使います。

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
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`provide` と `inject` が `'user'`、`'theme'` のような文字列キーを使い、同じ綴りを使うほかの provider と衝突し得ます。

```vue
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

## 良い

共有する `UserKey` を `Symbol` で作り、`InjectionKey<User>` の型を付けます。両呼び出しに同じキーを渡し、文字列リテラルを使わないようにします。

```vue
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [全ルール](../all.md)
