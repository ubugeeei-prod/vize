---
title: "css/no-id-selectors"
---

# `css/no-id-selectors`

詳細度が高い CSS の ID セレクターを検出します。

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
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

`#submit` で ID セレクターにスタイルを結び付けています。

```vue
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

## 良い

再利用できる `.submit` のクラスを使い、ID セレクターを取り除きます。

```vue
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [全ルール](../all.md)
