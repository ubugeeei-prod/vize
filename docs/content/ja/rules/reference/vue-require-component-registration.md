---
title: "vue/require-component-registration"
description: "使用するコンポーネントを import または登録します。"
---

# `vue/require-component-registration`

使用するコンポーネントを import または登録します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated` | なし | [ルールのオプション](../options.md) |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

アプリケーションのプラグインや Musea の previewSetup が登録するコンポーネント名を明示します。PascalCase と kebab-case を許可します。globals は名前で、正規表現は解釈しません。ignorePatterns には eslint-plugin-vue の vue/no-undef-components と同じく正規表現を指定します。静的な `is` 属性は `<component>` 上か `vue:` 接頭辞付きのときだけコンポーネント名として扱い、ほかの要素ではカスタマイズド組み込み要素なので検査しません。オプションだけではルールは有効になりません。後の設定は一覧全体を置き換え、空の一覧は継承した名前を消します。

## 悪い例

`MissingWidget` は登録されておらず、設定したグローバルコンポーネント一覧にもありません。

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

## 良い例

`MyButton` は例の `globals` に含まれます。既知のグローバルコンポーネントを検査対象から除く設定で、import や登録そのものは行いません。

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-registration": "warn"
    },
    "ruleOptions": {
      "vue/require-component-registration": {
        "globals": [
          "MyButton",
          "MyIcon"
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
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L59)
