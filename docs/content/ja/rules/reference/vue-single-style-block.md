---
title: "vue/single-style-block"
description: "SFC の style を一つのブロックにまとめます。"
---

# `vue/single-style-block`

SFC の style を一つのブロックにまとめます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

panel と title の scoped スタイルを二つの style ブロックに分けています。

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

## 良い例

両方のセレクタを一つの scoped style にまとめ、どちらのスタイルも残します。

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41)
