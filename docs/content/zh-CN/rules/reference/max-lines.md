---
title: "max-lines"
description: "限制文件的行数。"
---

# `max-lines`

限制文件的行数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

文件的行数超过配置的上限。

```vue annotate="remove:4,6,7,8,9,10,11,12,13,14,15,16,17"
<script setup lang="ts">
const label = "a";
</script>

<template>
  <ul>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
    <li>{{ label }}</li>
  </ul>
</template>

```

## 正确示例

文件的行数保持在配置的上限以内。

```vue annotate="add:5"
<script setup lang="ts">
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>

```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "max-lines": "warn"
    },
    "ruleOptions": {
      "max-lines": {
        "max": 10
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/max_lines.rs#L14)
