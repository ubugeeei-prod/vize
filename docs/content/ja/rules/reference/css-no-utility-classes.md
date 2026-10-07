---
title: "css/no-utility-classes"
---

# `css/no-utility-classes`

コンポーネント内で utility class を定義する箇所を検出します。

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: SFC の style ブロック内の CSS  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

## 良い

```vue
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [全ルール](../all.md)
