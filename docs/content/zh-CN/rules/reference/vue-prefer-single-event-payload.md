---
title: "vue/prefer-single-event-payload"
description: "强制自定义事件只传递单个负载参数。"
---

# `vue/prefer-single-event-payload`

强制自定义事件只传递单个负载参数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

move 事件声明并发出两个负载值，内联监听器只能收到 x。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [x: number, y: number] }>()
emit("move", 1, 2)
</script>
<template />
```

## 正确示例

move 事件传递一个对象，所有监听器都能收到两个坐标。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ move: [point: { x: number; y: number }] }>()
emit("move", { x: 1, y: 2 })
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
      "vue/prefer-single-event-payload": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/event_payload.rs#L34)
