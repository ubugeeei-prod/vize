---
title: "musea/require-component"
---

# `musea/require-component`

art ブロックに対象の component を指定します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
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
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

art に title はありますが、プレビューする component の指定がありません。

```vue
<art title="Button">
  <variant name="primary" />
</art>
```

## 良い

defineArt で ./Button.vue を art の component として指定します。

```vue
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [全ルール](../all.md)
