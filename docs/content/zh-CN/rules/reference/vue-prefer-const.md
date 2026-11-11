---
title: "vue/prefer-const"
description: "要求脚本和模板都不会重新赋值的绑定使用 const。"
---

# `vue/prefer-const`

要求脚本和模板都不会重新赋值的绑定使用 const。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板只读取 label，脚本和模板都没有重新赋值，因此不需要 let。

```vue annotate="remove:2"
<script setup>
let label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## 正确示例

脚本和模板只读取的绑定使用 const。

```vue annotate="add:2"
<script setup>
const label = "Hello";
</script>
<template><p>{{ label }}</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-const": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/prefer_const.rs#L16)
