---
title: "vue/no-restricted-v-bind"
description: "制限対象の v-bind 引数を検出します。"
---

# `vue/no-restricted-v-bind`

制限対象の v-bind 引数を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

`:v-focus` はディレクティブ風の引数を属性としてバインドしています。

```vue annotate="remove:2"
<template>
  <input :v-focus="enabled" />
</template>
```

## 良い例

`v-focus` をディレクティブ構文として直接使います。

```vue annotate="add:2"
<template>
  <input v-focus="enabled" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-bind": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/no_restricted_v_bind.rs#L9)
