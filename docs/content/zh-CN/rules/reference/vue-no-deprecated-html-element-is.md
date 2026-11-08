---
title: "vue/no-deprecated-html-element-is"
description: "禁止在原生 HTML 元素上使用已弃用的 `is` 属性。"
---

# `vue/no-deprecated-html-element-is`

禁止在原生 HTML 元素上使用已弃用的 `is` 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

原生 div 使用了没有前缀的旧 is 属性来指定 Vue 组件。

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

## 正确示例

动态组件使用 :is；原生元素的写法则显式使用 vue: 前缀。

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-html-element-is": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39)
