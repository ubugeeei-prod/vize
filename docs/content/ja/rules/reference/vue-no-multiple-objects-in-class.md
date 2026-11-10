---
title: "vue/no-multiple-objects-in-class"
description: ":class 配列内の複数のオブジェクト指定をまとめます。"
---

# `vue/no-multiple-objects-in-class`

:class 配列内の複数のオブジェクト指定をまとめます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。

## 悪い例

class の配列に、まとめられるオブジェクトリテラルを二つ直接指定しています。

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

## 良い例

条件を一つのオブジェクトにまとめます。オブジェクト一つと文字列の組み合わせや、リテラル以外の要素を持つ配列も許可されます。

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-multiple-objects-in-class": "warn"
    }
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L36)
