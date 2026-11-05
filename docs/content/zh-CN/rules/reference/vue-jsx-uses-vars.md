---
title: "vue/jsx-uses-vars"
description: "将 JSX 元素名称中引用的变量视为已使用。"
---

# `vue/jsx-uses-vars`

将 JSX 元素名称中引用的变量视为已使用。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

错误示例的诊断：`vue/no-unused-components`

这个辅助规则本身不会发出 `vue/jsx-uses-vars` 诊断。它会在 `vue/no-unused-components` 判断组件是否已使用时，将 JSX 标签绑定计为已使用。本示例需要同时启用这两条规则。`<MyButton.Part />` 将根绑定 `MyButton` 计为已使用；带命名空间的标签和 `this` 成员不提供导入的组件绑定。

## 错误示例

`MyButton` 已导入，但模板和 JSX 都没有引用它，因此配合使用的 `vue/no-unused-components` 会报告它。

```vue annotate="remove:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <div />;
</script>
<template><div /></template>
```

## 正确示例

JSX 标签引用了导入的 `MyButton`；辅助规则会记录这次使用，配合使用的规则便不再报告未使用的组件。

```vue annotate="add:3"
<script setup lang="jsx">
import MyButton from "./MyButton.vue";
const view = () => <MyButton />;
</script>
<template><div /></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/jsx-uses-vars": "error",
        "vue/no-unused-components": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/jsx_uses_vars.rs#L36)
