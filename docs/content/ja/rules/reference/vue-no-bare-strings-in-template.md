---
title: "vue/no-bare-strings-in-template"
description: "テンプレートに直接書かれた、国際化の対象となるテキストを検出します。"
---

# `vue/no-bare-strings-in-template`

テンプレートに直接書かれた、国際化の対象となるテキストを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

allowlist を指定すると、eslint-plugin-vue の既定の記号一覧を置き換えます。各項目はリテラルな部分文字列または /pattern/flags 形式で、残りのテキストを検査する前に除外します。既定値を保つには allowlist を省略し、[] で既定値を消します。オプションだけではルールを有効にしません。

## 悪い例

表示するテキストに、翻訳されない文字列を直接書いています。

```vue annotate="remove:2"
<template>
<p>Translate me</p>
</template>
```

## 良い例

設定した allowlist により、1 文字のキーラベル `W` を許可します。

```vue annotate="add:2"
<template>
<kbd>W</kbd>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-bare-strings-in-template": "warn"
    },
    "ruleOptions": {
      "vue/no-bare-strings-in-template": {
        "allowlist": [
          "/^[DRWXYmvx]$/u"
        ]
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [ルールのオプション](../options.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L51)
