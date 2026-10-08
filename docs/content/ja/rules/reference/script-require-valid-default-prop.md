---
title: "script/require-valid-default-prop"
---

# `script/require-valid-default-prop`

prop の default を宣言した型に合う値にします。

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
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

Number と Boolean の props に型の合わないスカラー既定値を付け、Array と Object には factory ではなく共有されるリテラル値を使っています。

```vue
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

## 良い

スカラーの既定値を `0` と `false` にし、配列とオブジェクトの既定値を新しい値を返す関数にします。`[String, Number]` の文字列既定値は、宣言した型の一つに合うため許可されます。

```vue
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [全ルール](../all.md)
