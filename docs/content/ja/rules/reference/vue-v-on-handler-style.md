---
title: "vue/v-on-handler-style"
description: "イベントハンドラーの書き方を参照または関数の形式に揃えます。"
---

# `vue/v-on-handler-style`

イベントハンドラーの書き方を参照または関数の形式に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

イベント属性の中に変更処理や複数の文を直接記述しています。

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

## 良い例

handler などの関数への参照を使うか、インライン処理が必要ならアロー関数や function 式で関数の境界を明示します。

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-handler-style": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33)
