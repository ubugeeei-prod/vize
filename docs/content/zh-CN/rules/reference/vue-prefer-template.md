---
title: "vue/prefer-template"
description: "优先使用模板字符串而非字符串拼接。"
---

# `vue/prefer-template`

优先使用模板字符串而非字符串拼接。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

字符串字面量与 `name` 被拼接。

```vue annotate="remove:2"
<template>
  <p>{{ "Hello " + name }}</p>
</template>
```

## 正确示例

模板字符串把插值保留在一个字符串中。

```vue annotate="add:2"
<template>
  <p>{{ `Hello ${name}` }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-template": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L145)
