---
title: "vue/no-v-html"
description: "警告使用 v-html，以避免 XSS 漏洞。"
---

# `vue/no-v-html`

警告使用 v-html，以避免 XSS 漏洞。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

v-html 将 content 解释为 HTML，而不是普通文本。

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

## 正确示例

双花括号插值将 content 显示为转义后的文本，而不注入 HTML。

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
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
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51)
