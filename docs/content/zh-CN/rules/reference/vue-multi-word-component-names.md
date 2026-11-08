---
title: "vue/multi-word-component-names"
description: "要求组件名称由多个单词组成。"
---

# `vue/multi-word-component-names`

要求组件名称由多个单词组成。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

问题在于文件名。需要重命名该组件；更改子标签不能修复此问题。

## 错误示例

Item.vue 使用了单个单词的组件名称。

`Item.vue`

```vue
<template><p>Item</p></template>
```

## 正确示例

TodoItem.vue 为同一模板使用了多个单词组成的组件名称。

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/multi-word-component-names": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
