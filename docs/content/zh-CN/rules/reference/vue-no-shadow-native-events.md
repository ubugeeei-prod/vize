---
title: "vue/no-shadow-native-events"
description: "禁止组件事件名与原生事件冲突。"
---

# `vue/no-shadow-native-events`

禁止组件事件名与原生事件冲突。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件声明的 `click` 与原生事件冲突。

```vue annotate="remove:2"
<script setup>
const emit = defineEmits(['click'])
</script>
<template />
```

## 正确示例

组件专用的 `activate` 事件避免冲突。

```vue annotate="add:2"
<script setup>
const emit = defineEmits(['activate'])
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
      "vue/no-shadow-native-events": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/events.rs#L28)
