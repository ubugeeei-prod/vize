---
title: "vue/require-component-is"
description: "要求 `<component>` 元素具有 `v-bind:is`。"
---

# `vue/require-component-is`

要求 `<component>` 元素具有 `v-bind:is`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

动态 `<component>` 没有 `is` 目标，因此 Vue 无法选择要渲染的组件。

```vue annotate="remove:2"
<template>
  <component />
</template>
```

## 正确示例

`:is="currentComponent"` 提供组件选择；该绑定可在运行时变化。

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-component-is": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L28)
