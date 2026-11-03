---
title: "vue/component-name-in-template-casing"
description: "要求模板中的组件名称使用指定的大小写风格。"
---

# `vue/component-name-in-template-casing`

要求模板中的组件名称使用指定的大小写风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

配置要求 PascalCase，但组件名称使用了 kebab-case 和 camelCase。

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

## 正确示例

MyComponent 使用 PascalCase；原生 slot 语法仍使用小写。

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31)
