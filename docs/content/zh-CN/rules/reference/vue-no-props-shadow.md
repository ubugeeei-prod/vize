---
title: "vue/no-props-shadow"
description: "禁止 script setup 声明遮蔽 prop。"
---

# `vue/no-props-shadow`

禁止 script setup 声明遮蔽 prop。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

本地 `value` 声明遮蔽了同名 prop。

```vue annotate="remove:3"
<script setup>
defineProps({ value: String })
const value = "local"
</script>
<template />
```

## 正确示例

本地绑定使用不同的名称。

```vue annotate="add:3"
<script setup>
defineProps({ value: String })
const localValue = "local"
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
      "vue/no-props-shadow": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L17)
