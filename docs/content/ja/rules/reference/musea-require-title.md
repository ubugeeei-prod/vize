---
title: "musea/require-title"
---

# `musea/require-title`

art ブロックに title を指定します。

既定の重大度: `error`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Musea の .art.vue ファイルの art / variant / style  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
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
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

## 良い

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [全ルール](../all.md)
