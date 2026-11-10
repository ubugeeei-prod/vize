---
title: "vue/dot-notation"
description: "对静态属性访问优先使用点号。"
---

# `vue/dot-notation`

对静态属性访问优先使用点号。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

静态的 `name` 属性使用了方括号访问。

```vue annotate="remove:2"
<template>
  <p>{{ user["name"] }}</p>
</template>
```

## 正确示例

点号直接表达同一个静态属性。

```vue annotate="add:2"
<template>
  <p>{{ user.name }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/dot-notation": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/dot_notation.rs#L10)
