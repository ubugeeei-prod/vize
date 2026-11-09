---
title: "vue/require-macro-variable-name"
description: "要求编译器宏使用约定的变量名。"
---

# `vue/require-macro-variable-name`

要求编译器宏使用约定的变量名。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`defineProps` 的结果绑定为 `p`。

```vue annotate="remove:2"
<script setup>
const p = defineProps({ value: String })
</script>
<template />
```

## 正确示例

约定的 `props` 名称标识宏结果。

```vue annotate="add:2"
<script setup>
const props = defineProps({ value: String })
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
      "vue/require-macro-variable-name": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc.rs#L56)
