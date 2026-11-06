---
title: "vue/valid-v-bind"
description: "要求 `v-bind` 指令有效。"
---

# `vue/valid-v-bind`

要求 `v-bind` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

不带参数的 `v-bind` 缺少对象表达式，空参数形式则缺少属性名称。

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

## 正确示例

提供属性与表达式、绑定对象，或使用 Vue 3.4+ 的同名简写，例如 `:loading`。

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-bind": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30)
