---
title: "vue/prefer-prop-type-boolean-first"
description: "prop の型コンストラクター配列で Boolean を先頭に置きます。"
---

# `vue/prefer-prop-type-boolean-first`

prop の型コンストラクター配列で Boolean を先頭に置きます。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

コンストラクターの順序を変えると、実行時の真偽属性の変換も変わる場合があります。この個別に有効化する検査は、実行時の `defineProps` と Options API の型コンストラクター配列が対象です。TypeScript の union 型を並べ替えるものではありません。

## 悪い例

`Boolean` より前に `String` があり、値を省略した `enabled` 属性が、真偽値を優先する規約とは異なる変換になります。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps({ enabled: [String, Boolean] });
</script>
```

## 良い例

文字列型も許可したまま `Boolean` を先頭に置き、真偽値を優先する属性変換にします。

```vue annotate="add:2"
<script setup lang="ts">
defineProps({ enabled: [Boolean, String] });
</script>
```

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-prop-type-boolean-first": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_prop_type_boolean_first.rs#L19)
