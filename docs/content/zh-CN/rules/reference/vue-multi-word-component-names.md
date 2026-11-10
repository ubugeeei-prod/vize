---
title: "vue/multi-word-component-names"
description: "要求组件名称由多个单词组成。"
---

# `vue/multi-word-component-names`

要求组件名称由多个单词组成。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

问题在于文件名。精确配置的 Item 例外不会允许其他单个单词的名称。

## 错误示例

Thing.vue 是单个单词的名称，且不在已配置的忽略列表中。

`Thing.vue`

```vue
<template><p>Item</p></template>
```

## 正确示例

Item.vue 由精确配置的 Item 例外允许。

`Item.vue`

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
    },
    "ruleOptions": {
      "vue/multi-word-component-names": {
        "ignores": [
          "Item"
        ]
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L41)
