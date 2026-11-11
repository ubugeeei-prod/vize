---
title: "vue/no-restricted-html-elements"
description: "禁止使用配置的元素。"
---

# `vue/no-restricted-html-elements`

禁止使用配置的元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板使用了被禁止的 marquee。

```vue annotate="remove:2"
<template>
  <marquee>x</marquee>
</template>
```

## 正确示例

模板使用允许的 p 元素。

```vue annotate="add:2"
<template>
  <p>x</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-html-elements": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-html-elements": [
        "marquee"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_html.rs#L13)
