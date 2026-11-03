---
title: "vapor/require-vapor-attribute"
description: "建议为 script setup 添加 vapor 属性。"
---

# `vapor/require-vapor-attribute`

建议为 script setup 添加 vapor 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 尚未在 SFC lint 中实现 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

> [!WARNING]
> 当前支持情况：`no-sfc-finding`。 Vize 接受在配置中使用这个规则 ID，但 SFC lint 目前还不会报告它。正确示例展示的是预期的写法。

此规则是具有空回调的占位项。添加 vapor 会选择 Vapor 编译；当前 linter 不会因缺少该属性而报告此目录 ID。

## 错误示例

script setup 块缺少 Vapor 编译属性。这是预期规范：当前为空的规则回调不会诊断它。

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## 正确示例

添加 vapor 会选择 Vapor 编译。这展示预期修复，并不表示当前 linter 会报告此目录规则。

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

## 可配置的 ID（当前没有 SFC 诊断）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vapor 规则](../vapor.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17)
