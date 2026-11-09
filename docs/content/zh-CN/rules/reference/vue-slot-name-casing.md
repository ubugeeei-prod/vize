---
title: "vue/slot-name-casing"
description: "要求渲染的插槽出口静态名称采用配置的大小写格式。"
---

# `vue/slot-name-casing`

要求渲染的插槽出口静态名称采用配置的大小写格式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

检查渲染的插槽出口的静态名称。父组件侧的 v-slot 和 #name 提供语法不在此规则范围内。

## 错误示例

渲染的插槽出口使用连字符静态名称 `my-slot`，而默认规范要求 camelCase。

```vue annotate="remove:2"
<template>
<MyCard><slot name="my-slot" /></MyCard>
</template>
```

## 正确示例

出口的静态名称使用 camelCase。父组件侧的 `#name` 提供语法不在此规则范围内。

```vue annotate="add:2"
<template>
<MyCard><slot name="mySlot" /></MyCard>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/slot-name-casing": "warn"
    },
    "ruleOptions": {
      "vue/slot-name-casing": "camelCase"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L29)
