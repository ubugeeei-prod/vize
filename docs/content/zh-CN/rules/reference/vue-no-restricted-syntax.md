---
title: "vue/no-restricted-syntax"
description: "禁止模板表达式中配置的语法。"
---

# `vue/no-restricted-syntax`

禁止模板表达式中配置的语法。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板中出现了配置限制的 `CallExpression`。

```vue annotate="remove:2"
<template>
  <p>{{ format(value) }}</p>
</template>
```

## 正确示例

模板读取已准备的绑定而不调用函数。

```vue annotate="add:2"
<template>
  <p>{{ formattedValue }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-syntax": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-syntax": {
        "selectors": [
          "CallExpression"
        ]
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/restricted.rs#L16)
