---
title: "vue/no-irregular-whitespace"
description: "检测模板表达式中的异常空白字符。"
---

# `vue/no-irregular-whitespace`

检测模板表达式中的异常空白字符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

表达式含有不换行空格。

```vue annotate="remove:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## 正确示例

表达式使用普通源码空格。

```vue annotate="add:2"
<template>
  <p>{{ value + other }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-irregular-whitespace": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_irregular_whitespace.rs#L10)
