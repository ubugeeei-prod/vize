---
title: "vue/valid-v-html"
description: "要求 `v-html` 指令有效。"
---

# `vue/valid-v-html`

要求 `v-html` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-html` 缺少表达式，或使用了此指令不支持的参数或修饰符。

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

## 正确示例

`v-html="html"` 提供有效表达式。语法有效并不会清理 HTML，也不会让不可信内容变得安全。

```vue annotate="add:2"
<template>
<div v-html="html"></div>
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
        "vue/valid-v-html": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28)
