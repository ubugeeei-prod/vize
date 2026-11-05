---
title: "script/no-deprecated-dollar-listeners-api"
description: "Vue 3 で $attrs に統合された $listeners を検出します。"
---

# `script/no-deprecated-dollar-listeners-api`

Vue 3 で $attrs に統合された $listeners を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

スクリプトが削除済みの `$listeners` を参照しています。テンプレートでもコンポーネントの `$listeners` を転送しています。所有元を確認できた同名の setup 変数やテンプレートローカル変数は別の値として扱います。

```vue annotate="remove:4,5,9"
<script>
export default {
  mounted() {
    const vm = this
    return vm.$listeners
  }
}
</script>
<template><Child v-on="$listeners" /></template>
```

## 良い例

`this.$attrs` と `v-bind="$attrs"` で Vue 3 の属性とリスナーを参照・転送します。スクリプト内の参照元のオブジェクトや参照方法に対する、既存の広い検出範囲は変わりません。

```vue annotate="add:4,8"
<script>
export default {
  mounted() {
    return this.$attrs
  }
}
</script>
<template><Child v-bind="$attrs" /></template>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L54)
