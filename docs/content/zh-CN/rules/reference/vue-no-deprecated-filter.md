---
title: "vue/no-deprecated-filter"
description: "禁止使用已弃用的 Vue 2 管道运算符过滤器语法。"
---

# `vue/no-deprecated-filter`

禁止使用已弃用的 Vue 2 管道运算符过滤器语法。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

管道使用了已移除的 Vue 过滤器语法来应用 capitalize。

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

## 正确示例

调用 capitalize(message)，以普通表达式完成转换。

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
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
        "vue/no-deprecated-filter": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L56)
