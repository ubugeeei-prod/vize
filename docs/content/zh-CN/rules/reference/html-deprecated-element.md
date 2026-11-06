---
title: "html/deprecated-element"
description: "禁止已弃用的 HTML 元素。"
---

# `html/deprecated-element`

禁止已弃用的 HTML 元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`center` 使用了已弃用的 HTML 呈现元素。

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

## 正确示例

section 和样式类替代弃用元素，同时保留内容。

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/deprecated-element": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [HTML 规则](../html.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33)
