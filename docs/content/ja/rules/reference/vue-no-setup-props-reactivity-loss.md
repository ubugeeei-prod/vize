---
title: "vue/no-setup-props-reactivity-loss"
description: "setup の props をコピーし、更新に追従しなくなる箇所を検出します。"
---

# `vue/no-setup-props-reactivity-loss`

setup の props をコピーし、更新に追従しなくなる箇所を検出します。

| 既定の重大度 | プリセット | 自動修正 | オプション |
| --- | --- | --- | --- |
| `warning` | なし | なし | なし |

適用範囲：Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。

Options API の `setup` と `<script setup>` のルートスコープで props の値をコピーする箇所を検査します。Vue 3.5 のリアクティブな props 分割代入そのものは許可し、その後で script のルートへ値をコピーする箇所を報告します。別の関数内での読み取りは許可します。

## 悪い例

script のルートで `props.count` をコピーすると、その時点の値が保持され、後からの prop の更新に追従しません。

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps({ count: Number });
const snapshot = props.count;
</script>
```

## 良い例

computed の getter 内で `props.count` を読み、派生する結果を prop の変化に追従させます。

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps({ count: Number });
const snapshot = computed(() => props.count);
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
        "vue/no-setup-props-reactivity-loss": "warn"
      }
    },
  },
});
```

設定したら `vp run lint` を実行します。

## 関連ページ

- [Vue ルール](../vue.md)
- [全ルール](../all.md)
- [ソースコード](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_setup_props_reactivity_loss.rs#L44)
