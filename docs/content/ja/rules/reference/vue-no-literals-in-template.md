---
title: "vue/no-literals-in-template"
description: "v-bind のオブジェクト・配列・関数リテラルを検出します。"
---

# `vue/no-literals-in-template`

v-bind のオブジェクト・配列・関数リテラルを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

オブジェクトリテラルが描画時に新しい値を作ります。

```vue annotate="remove:2"
<template>
  <Widget :options="{ eager: true }" />
</template>
```

## 良い例

script の安定したバインディングから options オブジェクトを渡します。

```vue annotate="add:2"
<template>
  <Widget :options="options" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-literals-in-template": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/no_literals_in_template.rs#L10)
