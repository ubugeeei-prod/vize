---
title: "vue/no-unused-setup-bindings"
description: "禁止从未读取的 script setup 绑定。"
---

# `vue/no-unused-setup-bindings`

禁止从未读取的 script setup 绑定。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

script setup 中的 message 绑定从未被模板读取。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

## 正确示例

段落插值使用 message，从而使用了声明的绑定。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-setup-bindings": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19)
