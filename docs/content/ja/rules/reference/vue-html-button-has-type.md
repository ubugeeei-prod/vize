---
title: "vue/html-button-has-type"
description: "button に有効な type を明示します。"
---

# `vue/html-button-has-type`

button に有効な type を明示します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

一つは type を省略し、もう一つは対応しない foo を指定しています。

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

## 良い例

button、submit、reset を明示します。バインドする type は動的な値として扱います。

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
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
        "vue/html-button-has-type": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39)
