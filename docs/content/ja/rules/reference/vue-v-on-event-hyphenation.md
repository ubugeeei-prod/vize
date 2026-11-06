---
title: "vue/v-on-event-hyphenation"
description: "コンポーネントのカスタムイベント名を設定した形式に揃えます。"
---

# `vue/v-on-event-hyphenation`

コンポーネントのカスタムイベント名を設定した形式に揃えます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | あり | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

カスタムコンポーネントのリスナーがハイフン区切りではなく `@myEvent` です。

```vue annotate="remove:2,3,4"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
<my-component @update:modelValue="handler" />
</template>
```

## 良い例

`@my-event` に変更します。例のネイティブ要素のリスナーと動的なイベント引数はこの検査の対象外です。

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-event-hyphenation": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L40)
