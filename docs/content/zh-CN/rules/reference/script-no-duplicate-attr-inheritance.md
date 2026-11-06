---
title: "script/no-duplicate-attr-inheritance"
description: "标记重复应用透传属性的组件。"
---

# `script/no-duplicate-attr-inheritance`

标记重复应用透传属性的组件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

显式 `inheritAttrs: true` 重复了 Vue 的默认值。即使没有展示根节点 `$attrs` 展开，规则也会报告此多余字面量。

```vue annotate="remove:1"
<script setup></script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## 正确示例

`inheritAttrs: false` 表达实际的退出选择，空选项对象则隐式保留默认继承。两者都不重复多余的 `true`。

```vue annotate="add:1"
<script setup>defineOptions({ inheritAttrs: false })</script>
<template><div><button v-bind="$attrs">Save</button></div></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-duplicate-attr-inheritance": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L35)
