---
title: "vue/valid-v-else-if"
description: "要求 `v-else-if` 指令有效。"
---

# `vue/valid-v-else-if`

要求 `v-else-if` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

示例中的 `v-else-if` 前面没有条件分支、缺少条件，或带有参数或修饰符。

```vue annotate="remove:2,3,4,5"
<template>
<div v-else-if="foo"></div> <!-- without preceding v-if -->
<div v-if="foo"></div><div v-else-if></div>
<div v-if="foo"></div><div v-else-if:arg="bar"></div>
<div v-if="foo"></div><div v-else-if.mod="bar"></div>
</template>
```

## 正确示例

将带条件的 `v-else-if` 紧跟在它所延续的 `v-if` 或 `v-else-if` 分支之后。

```vue annotate="add:2,3"
<template>
<div v-if="foo"></div>
<div v-else-if="bar"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-else-if": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else_if.rs#L34)
