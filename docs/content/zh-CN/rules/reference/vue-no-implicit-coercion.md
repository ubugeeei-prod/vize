---
title: "vue/no-implicit-coercion"
description: "禁止模板表达式中的隐式类型转换。"
---

# `vue/no-implicit-coercion`

禁止模板表达式中的隐式类型转换。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

双重否定隐式转换了值。

```vue annotate="remove:2"
<template>
  <p>{{ !!value }}</p>
</template>
```

## 正确示例

`Boolean` 明确表达转换。

```vue annotate="add:2"
<template>
  <p>{{ Boolean(value) }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-implicit-coercion": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template.rs#L118)
