---
title: "vue/warn-custom-directive"
description: "登録が必要なカスタムディレクティブを検出します。"
---

# `vue/warn-custom-directive`

登録が必要なカスタムディレクティブを検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

グローバル登録したディレクティブ名を v- 接頭辞なしの完全一致で指定します。script setup のローカルな vName 束縛は引き続き自動検出します。オプションだけではルールを有効にしません。後の設定レイヤーは一覧全体を置き換え、[] で継承した名前を消します。

## 悪い例

`v-focus` はプロジェクト固有のディレクティブ実装を必要とするため、この任意の規約で報告されます。

```vue annotate="remove:2"
<template>
<button type="button" v-focus>Focus</button>
</template>
```

## 良い例

`v-show` は Vue の組み込みディレクティブなので、プロジェクトでの登録は不要です。

```vue annotate="add:2"
<template>
<button type="button" v-show="visible">Copy</button>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/warn-custom-directive": "warn"
    },
    "ruleOptions": {
      "vue/warn-custom-directive": {
        "ignore": [
          "hover-tooltip"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44)
