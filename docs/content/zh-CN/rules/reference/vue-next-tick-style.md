---
title: "vue/next-tick-style"
description: "要求以 Promise 方式使用 nextTick。"
---

# `vue/next-tick-style`

要求以 Promise 方式使用 nextTick。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

回调被传给了 `nextTick`。

```vue annotate="remove:3"
<script setup>
import { nextTick } from "vue"
nextTick(() => focusInput())
</script>
<template />
```

## 正确示例

代码等待 Promise 后再继续。

```vue annotate="add:3,4"
<script setup>
import { nextTick } from "vue"
await nextTick()
focusInput()
</script>
<template />
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/next-tick-style": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/next_tick_style.rs#L13)
