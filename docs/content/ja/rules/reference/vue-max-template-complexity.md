---
title: "vue/max-template-complexity"
---

# `vue/max-template-complexity`

コンポーネント自身のテンプレートの複雑度を制限します。

[悪い例](#悪い) · [良い例](#良い)

既定の重大度: `warning`  
プリセット: _none_  
自動修正: なし。修正内容を確認してください  
適用範囲: Vue SFC のテンプレートと各ブロック。ルールに必要な script の文脈も例に含めています。  
オプション: ルール固有のオプションはありません。重大度とプリセットは設定できます。

悪い例のサイクロマティック複雑度は 13、認知的複雑度は 25 で、閾値 11 と 16 を超えます。コンポーネント単位で計測し、対象はインライン HTML テンプレートです。

例の二つの値の計算内訳は[複雑度の計算とコンポーネントの境界](../../guide/cross-file-complexity.md)を参照してください。

## 設定（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/max-template-complexity": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

## 悪い

親のテンプレートに記述した分岐、ループ、スロットの内容、式内の条件分岐により、サイクロマティック複雑度が 13、認知的複雑度が 25 となり、既定の閾値 11 と 16 を超えています。

```vue
<script setup lang="ts">
defineProps<{ rows: Row[] }>();
</script>
<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

## 良い

描画を RowList に分け、親には v-if を一つ残します。親自身のサイクロマティック複雑度は 2、認知的複雑度は 1 になります。

```vue
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

良い例は上記の設定でこのルールの診断が出ない例です。他のルールの診断は出る場合があります。

[実装](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [全ルール](../all.md)
