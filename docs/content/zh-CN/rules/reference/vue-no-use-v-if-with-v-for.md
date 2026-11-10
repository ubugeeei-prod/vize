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

同一个列表元素同时使用 v-if 和 v-for，但条件没有引用循环变量。

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="showAll" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

## 正确示例

启用 allowUsingIterationVar 后，条件可以引用同一个 v-for 中的 item。

```vue annotate="add:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
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

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35)
