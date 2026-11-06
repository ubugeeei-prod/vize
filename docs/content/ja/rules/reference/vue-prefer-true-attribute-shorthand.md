---
title: "vue/prefer-true-attribute-shorthand"
description: "true を指定するバインディングを真偽属性の省略形にします。"
---

# `vue/prefer-true-attribute-shorthand`

true を指定するバインディングを真偽属性の省略形にします。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

標準の真偽属性 disabled に、固定の true をバインドしています。

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

## 良い例

標準の真偽属性は省略形にします。false のバインディングやコンポーネントの prop は値を残します。

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-true-attribute-shorthand": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38)
