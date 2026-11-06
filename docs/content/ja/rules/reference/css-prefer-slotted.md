---
title: "css/prefer-slotted"
description: "slot や子コンポーネントに対する scoped CSS のセレクターを検査します。"
---

# `css/prefer-slotted`

slot や子コンポーネントに対する scoped CSS のセレクターを検査します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：プレーン CSS の style ブロック（scss・sass・less・stylus のブロックは対象外）。

## 悪い例

scoped の CSS で、スロットに渡された要素ではなく、スロットの挿入位置を表す slot を対象にしています。

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

## 良い例

:slotted(.label) を使い、スロットに渡された .label クラスを持つ要素を対象にします。

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
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
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [CSS ルール](../css.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34)
