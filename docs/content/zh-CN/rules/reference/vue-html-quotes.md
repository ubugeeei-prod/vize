---
title: "vue/html-quotes"
description: "规范 HTML 属性的引号风格。"
---

# `vue/html-quotes`

规范 HTML 属性的引号风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

属性使用单引号或不加引号，不符合双引号规范。

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

## 正确示例

普通属性和指令表达式都使用双引号。

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-quotes": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53)
