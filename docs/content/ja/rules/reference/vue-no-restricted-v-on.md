---
title: "vue/no-restricted-v-on"
description: "設定した v-on 引数を使う箇所を検出します。"
---

# `vue/no-restricted-v-on`

設定した v-on 引数を使う箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

ボタンが禁止された legacy-click を購読しています。

```vue annotate="remove:2"
<template>
  <button type="button" @legacy-click="save">x</button>
</template>
```

## 良い例

ボタンは click を購読します。

```vue annotate="add:2"
<template>
  <button type="button" @click="save">x</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-v-on": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-v-on": [
        "legacy-click"
      ]
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_v_on.rs#L13)
