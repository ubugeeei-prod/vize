---
title: "vue/v-on-style"
description: "规范 `v-on` 指令风格。"
---

# `vue/v-on-style`

规范 `v-on` 指令风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-on:click` 使用完整的事件监听形式，但规则要求简写。

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

## 正确示例

`@click` 使用配置的简写，并保留同一处理器。

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-style": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L29)
