---
title: "vue/max-template-complexity"
description: "限制组件自身模板的复杂度（圈复杂度和认知复杂度）。"
---

# `vue/max-template-complexity`

限制组件自身模板的复杂度（圈复杂度和认知复杂度）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

Bad 的圈复杂度为 13，认知复杂度为 25（上限分别为 11 和 16）。每个组件单独计量；仅支持内联 HTML 模板。

示例中两个分数的计算方式见[复杂度计分与组件边界](../../guide/cross-file-complexity.md)。

## 错误示例

父组件编写的分支、循环、插槽内容和表达式中的判断产生了 13 和 25 的复杂度，超过默认上限 11 和 16。

```vue annotate="remove:1,2,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19"
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

## 正确示例

父组件模板将渲染交给 RowList，只保留一个 v-if；其自身的复杂度为 2 和 1。

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-template-complexity": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56)
