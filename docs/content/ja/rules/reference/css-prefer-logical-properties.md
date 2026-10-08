---
title: "css/prefer-logical-properties"
---

# `css/prefer-logical-properties`

書字方向に対応する CSS の論理プロパティを使います。

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
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

margin-left は文字の方向に関係なく物理的な左側を指定します。

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

## 良い

margin-inline-start を使い、インライン方向の開始側に余白を指定します。

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [全ルール](../all.md)
