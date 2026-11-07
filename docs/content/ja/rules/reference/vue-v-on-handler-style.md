---
title: "vue/v-on-handler-style"
description: "イベントハンドラーをメソッド参照の形式に揃えます。"
---

# `vue/v-on-handler-style`

イベントハンドラーをメソッド参照の形式に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

再利用可能なメソッド参照ではなく、インライン関数・変更処理・複数の文をイベント属性に記述しています。

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

## 良い例

handler などの関数への参照を使います。インライン処理は名前付きメソッドへ移し、テンプレートでは安定したハンドラーを参照します。

```vue annotate="add:2,3"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-handler-style": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L31)
