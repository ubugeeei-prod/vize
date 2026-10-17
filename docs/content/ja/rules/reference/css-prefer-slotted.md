---
title: "css/prefer-slotted"
---

# `css/prefer-slotted`

slot や子コンポーネントに対する scoped CSS のセレクターを検査します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

scoped の CSS で、スロットに渡された要素ではなく、スロットの挿入位置を表す slot を対象にしています。

```vue
<style scoped>
slot { color: red; }
</style>
```

## 良い

:slotted(.label) を使い、スロットに渡された .label クラスを持つ要素を対象にします。

```vue
<style scoped>
:slotted(.label) { color: red; }
</style>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [全ルール](../all.md)
