---
title: "script/no-deep-destructure-in-props"
---

# `script/no-deep-destructure-in-props`

defineProps の深い分割代入を検出します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: `opinionated`, `nuxt`  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC の JavaScript / TypeScript の script ブロック。対象となる Options API または script setup の形式は例を参照してください。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

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

```sh
vp run lint
```

## 悪い

唯一の inline `<script setup>` のトップレベルで、コンパイラーマクロの props の `user` からさらに `name` を取り出し、既定の深さ 1 を超えています。オブジェクトの内側の配列も深さに数えます。Opinionated と Nuxt では、深さの方針を検査するこの警告が既定で有効です。他のプリセットでは個別に有効化します。コンパイラーの `propsDestructure` 設定は参照せず、`withDefaults` でコンパイルできる場合も警告します。

```vue
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

## 良い

props オブジェクトを保ち、computed の getter で `props.user.name` を参照します。深い代入パターンを使わず、入れ子の参照を明示します。

```vue
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L39) · [全ルール](../all.md)
