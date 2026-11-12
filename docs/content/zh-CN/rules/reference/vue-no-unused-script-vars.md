---
title: "vue/no-unused-script-vars"
description: "禁止未使用的脚本变量、导入和参数。"
---

# `vue/no-unused-script-vars`

禁止未使用的脚本变量、导入和参数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

script setup 声明了 unused，脚本和模板都没有读取它。

```vue annotate="remove:4"
<script setup>
const unused = 1;
</script>
<template><p>Welcome</p></template>
```

## 正确示例

模板插值读取 unused，因此声明的绑定被使用了。

```vue annotate="add:4"
<script setup>
const unused = 1;
</script>
<template><p>{{ unused }}</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-script-vars": "warn"
    },
    "ruleOptions": {
      "vue/no-unused-script-vars": {
        "varsIgnorePattern": "^_",
        "argsIgnorePattern": "^_"
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/script_unused/no_unused_script_vars.rs#L23)
