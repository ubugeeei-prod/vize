---
title: "vue/require-toggle-inside-transition"
description: "transition の子要素に表示を切り替える条件を指定します。"
---

# `vue/require-toggle-inside-transition`

transition の子要素に表示を切り替える条件を指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`<Transition>` 内の静的な子に表示の切り替えや動的な選択がなく、enter / leave が発生する条件がありません。

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

## 良い例

`v-if="show"` で子の有無を切り替え、enter / leave の対象にします。

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-toggle-inside-transition": "error"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L49)
