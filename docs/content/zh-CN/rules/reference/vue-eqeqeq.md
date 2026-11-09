---
title: "vue/eqeqeq"
description: "要求模板表达式使用严格相等运算符。"
---

# `vue/eqeqeq`

要求模板表达式使用严格相等运算符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

宽松相等可能转换任一操作数。

```vue annotate="remove:2"
<template>
  <p>{{ value == null }}</p>
</template>
```

## 正确示例

严格相等不进行类型转换。

```vue annotate="add:2"
<template>
  <p>{{ value === null }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/eqeqeq": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L68)
