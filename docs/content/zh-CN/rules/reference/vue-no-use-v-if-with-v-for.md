---
title: "vue/no-use-v-if-with-v-for"
description: "禁止在同一元素上使用 `v-if` 与 `v-for`。"
---

# `vue/no-use-v-if-with-v-for`

禁止在同一元素上使用 `v-if` 与 `v-for`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

同一个列表元素同时使用 v-if 和 v-for，并通过循环绑定检查可见性。

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## 正确示例

计算属性中的集合先筛选出可见项，再由模板进行迭代。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-use-v-if-with-v-for": "warn"
    },
    "ruleOptions": {
      "vue/no-use-v-if-with-v-for": {
        "allowUsingIterationVar": true
      }
    }
  },
});
```

然后运行 `vp run lint`。

`allowUsingIterationVar` 默认为 `false`。设为 `true` 后，仅当同一元素的 `v-if` 条件引用
对应 `v-for` 的迭代变量时才允许；无关条件仍会报告。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35)
