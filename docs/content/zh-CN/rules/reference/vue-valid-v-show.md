---
title: "vue/valid-v-show"
description: "要求 `v-show` 指令有效。"
---

# `vue/valid-v-show`

要求 `v-show` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-show` 缺少可见性表达式，或被放在 `<template>` 上；该元素没有可改变 display 的 DOM 元素。

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

## 正确示例

将可见性表达式应用于 `<div>` 等实际渲染的元素。

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
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
        "vue/valid-v-show": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28)
