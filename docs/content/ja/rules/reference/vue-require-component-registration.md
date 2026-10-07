---
title: "vue/require-component-registration"
---

# `vue/require-component-registration`

使用するコンポーネントを import または登録します。

既定の重大度: `warning`  
プリセット: `opinionated`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレート・ブロック。必要な script の文脈も例に含めています。  
オプション: [型付きオプションと既定値](../options.md)を参照してください。

application plugin や Musea previewSetup が登録する component 名を明示します。PascalCase と kebab-case を許可し、正規表現は解釈しません。option だけではルールは有効になりません。後の設定は list 全体を置き換え、空 list は継承した名前を消します。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
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
  },
});
```

```sh
vp run lint
```

## 悪い

```vue
<template>
<MissingWidget />
</template>
```

## 良い

```vue
<template>
<MyButton />
</template>
```

良い例は上記の設定でこのルールの検出を避ける例です。他のルールでは検出される場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [全ルール](../all.md)
