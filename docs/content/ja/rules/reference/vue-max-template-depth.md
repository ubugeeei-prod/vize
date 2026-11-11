---
title: "vue/max-template-depth"
description: "テンプレートの入れ子が上限を超えた箇所を検出します。"
---

# `vue/max-template-depth`

テンプレートの入れ子が上限を超えた箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

span の入れ子が設定した上限を超えています。

```vue annotate="remove:2,3,4,5,6,7,8"
<template>
  <div>
    <section>
      <div>
        <span>deep</span>
      </div>
    </section>
  </div>
</template>
```

## 良い例

テンプレートは要素 1 つです。

```vue annotate="add:2"
<template>
  <p>ok</p>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-template-depth": "warn"
    },
    "ruleOptions": {
      "vue/max-template-depth": {
        "maxDepth": 3
      }
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_template_depth.rs#L13)
