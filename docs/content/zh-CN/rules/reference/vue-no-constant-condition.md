---
title: "vue/no-constant-condition"
description: "禁止模板表达式中的常量条件。"
---

# `vue/no-constant-condition`

禁止模板表达式中的常量条件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

三元表达式的条件始终为真。

```vue annotate="remove:2"
<template>
  <p>{{ true ? primary : fallback }}</p>
</template>
```

## 正确示例

分支取决于响应式的 `ready` 值。

```vue annotate="add:2"
<template>
  <p>{{ ready ? primary : fallback }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-constant-condition": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_constant_condition.rs#L10)
