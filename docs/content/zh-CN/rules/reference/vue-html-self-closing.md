---
title: "vue/html-self-closing"
description: "规范自闭合标签风格。"
---

# `vue/html-self-closing`

规范自闭合标签风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

空组件使用成对的起止标签，而空元素 img 和 br 没有采用配置要求的自闭合写法。

```vue annotate="remove:3,4"
<template>
  <section>
    <MyComponent></MyComponent>
    <div></div>
  </section>
</template>
```

## 正确示例

组件和空元素使用自闭合语法；包含内容的 div 保留结束标签。

```vue annotate="add:3,4,5,6,7"
<template>
  <section>
    <MyComponent />
    <div />
    <img>
    <br>
    <div>content</div>
  </section>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/html-self-closing": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L31)
