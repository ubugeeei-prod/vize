---
title: "vue/no-bare-strings-in-template"
description: "テンプレートに直接書かれた、国際化の対象となるテキストを検出します。"
---

# `vue/no-bare-strings-in-template`

テンプレートに直接書かれた、国際化の対象となるテキストを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

表示するテキストや利用者向けの説明を指定する属性に、翻訳されない文字列を直接書いています。

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

## 良い例

翻訳する内容を $t で取得します。記号だけ、数値だけの内容は例外です。

```vue annotate="add:2,3,4,5"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<button :title="$t('close')">{{ $t('x') }}</button>
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
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L50)
