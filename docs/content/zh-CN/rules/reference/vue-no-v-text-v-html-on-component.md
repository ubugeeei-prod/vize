---
title: "vue/no-v-text-v-html-on-component"
description: "禁止在组件元素上使用 v-text / v-html。"
---

# `vue/no-v-text-v-html-on-component`

禁止在组件元素上使用 v-text / v-html。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件标签使用 v-html 或 v-text，这些指令会替换元素内容，而不是提供组件插槽。

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

## 正确示例

原生 HTML 目标可接收这些指令；MyComponent 通过默认插槽接收内容。

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-text-v-html-on-component": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L34)
