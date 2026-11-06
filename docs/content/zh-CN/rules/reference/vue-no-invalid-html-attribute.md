---
title: "vue/no-invalid-html-attribute"
description: "禁止 HTML 属性使用无效的静态值。"
---

# `vue/no-invalid-html-attribute`

禁止 HTML 属性使用无效的静态值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

锚点元素将 stylesheet 用作 rel 值，但该值应当用于样式表 link 元素。

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

## 正确示例

锚点使用 help，这是适用于所链接帮助资源的 rel 值。

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-invalid-html-attribute": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12)
