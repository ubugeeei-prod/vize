---
title: "vue/no-deprecated-functional-template"
description: "禁止在 SFC 的 `<template>` 上使用 `functional` 属性。"
---

# `vue/no-deprecated-functional-template`

禁止在 SFC 的 `<template>` 上使用 `functional` 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

SFC 模板使用了已移除的 functional 属性，并读取旧的 props 上下文。

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

## 正确示例

普通模板省略 functional，直接读取组件绑定 msg。

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57)
