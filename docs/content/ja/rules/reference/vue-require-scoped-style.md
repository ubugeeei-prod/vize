---
title: "vue/require-scoped-style"
description: "style に scoped を指定する方針を適用します。"
---

# `vue/require-scoped-style`

style に scoped を指定する方針を適用します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`.button` の style にスコープがなく、他のコンポーネントの一致する要素にも作用します。

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

## 良い例

`scoped` を style に付け、同じセレクタと宣言に Vue のコンポーネントスコープを適用します。

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
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
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49)
