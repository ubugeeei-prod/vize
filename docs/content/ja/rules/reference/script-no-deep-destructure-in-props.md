---
title: "script/no-deep-destructure-in-props"
description: "defineProps の深い分割代入を検出します。"
---

# `script/no-deep-destructure-in-props`

defineProps の深い分割代入を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

## 悪い例

この SFC 内に一つだけある、内容を直接記述した `<script setup>` のトップレベルで、コンパイラーマクロの props の `user` からさらに `name` を取り出し、既定の深さ 1 を超えています。オブジェクトの内側の配列も深さに数えます。Opinionated と Nuxt では、深さの方針を検査するこの警告が既定で有効です。他のプリセットでは個別に有効化します。コンパイラーの `propsDestructure` 設定は参照せず、`withDefaults` でコンパイルできる場合も警告します。

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

## 良い例

props オブジェクトを保ち、computed の getter で `props.user.name` を参照します。深い代入パターンを使わず、入れ子の参照を明示します。

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
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
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [型と script のルール](../type-and-script.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39)
