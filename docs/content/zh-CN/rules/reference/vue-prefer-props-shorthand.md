---
title: "vue/prefer-props-shorthand"
description: "建议使用 props 简写语法（Vue 3.4+）。"
---

# `vue/prefer-props-shorthand`

建议使用 props 简写语法（Vue 3.4+）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

每个绑定都重复了对应的变量名，包括与连字符参数对应的驼峰名称。

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

## 正确示例

Vue 3.4+ 的同名绑定简写省略了重复表达式；bar 等不同的来源变量仍显式写出。

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-props-shorthand": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L42)
