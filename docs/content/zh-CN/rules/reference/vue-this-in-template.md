---
title: "vue/this-in-template"
description: "禁止在模板表达式中使用 `this.`。"
---

# `vue/this-in-template`

禁止在模板表达式中使用 `this.`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板表达式显式访问 `this.message`、`this.className` 和 `this.handleClick`，但 Vue 已直接暴露这些绑定。

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

## 正确示例

直接使用 `message`、`className` 和 `handleClick`。字面字符串 `'this.is.a.string'` 保持不变，因为它不是成员访问。

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
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
        "vue/this-in-template": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L37)
