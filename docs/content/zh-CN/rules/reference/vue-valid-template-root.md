---
title: "vue/valid-template-root"
description: "要求 `<template>` 根节点符合 Vue 3 片段语义。"
---

# `vue/valid-template-root`

要求 `<template>` 根节点符合 Vue 3 片段语义。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

普通的嵌套 `<template>` 占据模板根部，却没有赋予它渲染作用的指令。

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

## 正确示例

`<div>` 是可渲染的根元素。此示例并未对 Vue 3 片段施加普遍的单根节点限制。

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-template-root": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82)
