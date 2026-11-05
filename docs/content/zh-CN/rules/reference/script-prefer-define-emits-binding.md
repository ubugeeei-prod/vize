---
title: "script/prefer-define-emits-binding"
description: "优先使用从 defineEmits 捕获的函数而不是模板 $emit。"
---

# `script/prefer-define-emits-binding`

优先使用从 defineEmits 捕获的函数而不是模板 $emit。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

模板调用组件的 `$emit` 辅助函数，但 `<script setup>` 已捕获 `defineEmits` 返回的类型化函数。

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="$emit('save', 1)">Save</button></template>
```

## 正确示例

模板改为调用捕获的 `emit` 绑定，使事件名和 payload 经过组件的 `defineEmits` 约定。

```vue annotate="add:4"
<script setup lang="ts">
const emit = defineEmits<{ save: [id: number] }>();
</script>
<template><button @click="emit('save', 1)">Save</button></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-emits-binding": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_define_emits_binding.rs#L16)
