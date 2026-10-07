---
title: "css/prefer-logical-properties"
---

# `css/prefer-logical-properties`

書字方向に対応する CSS の論理プロパティを使います。

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

```vue
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

## 良い

```vue
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [全ルール](../all.md)
