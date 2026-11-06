---
title: "vue/v-bind-style"
description: "规范 `v-bind` 指令风格。"
---

# `vue/v-bind-style`

规范 `v-bind` 指令风格。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-bind:class` 使用了完整形式，但配置的绑定风格要求冒号简写。

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

## 正确示例

`:class` 使用所需的简写，并保留相同表达式；此规则检查写法，而非值的类型。

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-bind-style": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L28)
