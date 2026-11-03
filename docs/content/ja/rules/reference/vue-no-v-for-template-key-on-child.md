---
title: "vue/no-v-for-template-key-on-child"
description: "template v-for の key を子ではなく template に指定します。"
---

# `vue/no-v-for-template-key-on-child`

template v-for の key を子ではなく template に指定します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

繰り返す template に key がなく、子の p に指定しています。

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

## 良い例

template の v-for に key を移し、繰り返すフラグメント全体を識別します。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
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
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30)
