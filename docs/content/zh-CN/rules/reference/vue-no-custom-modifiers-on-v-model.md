---
title: "vue/no-custom-modifiers-on-v-model"
description: "为兼容 Vue 2，禁止在组件的 `v-model` 上使用自定义修饰符。"
---

# `vue/no-custom-modifiers-on-v-model`

为兼容 Vue 2，禁止在组件的 `v-model` 上使用自定义修饰符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

这条按需启用的规则对组件 `v-model` 施加 Vue 2 兼容性约定。只允许 `lazy`、`number` 和 `trim` 修饰符。Vue 3 组件可以实现自定义修饰符，因此 Vue 3 预设不启用此限制。原生元素不在这条规则的检查范围内。

## 错误示例

组件的 `.capitalize` 是自定义修饰符，违反了这里的 Vue 2 兼容性限制。

```vue annotate="remove:1"
<template><MyField v-model.capitalize="value" /></template>
```

## 正确示例

使用不带自定义修饰符的普通组件 `v-model` 约定；如果需要首字母大写，应显式实现该处理。

```vue annotate="add:1"
<template><MyField v-model="value" /></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-custom-modifiers-on-v-model": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_custom_modifiers_on_v_model.rs#L23)
