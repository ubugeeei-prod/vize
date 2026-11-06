---
title: "vue/no-deprecated-inline-template"
description: "禁止使用已弃用的 `inline-template` 属性。"
---

# `vue/no-deprecated-inline-template`

禁止使用已弃用的 `inline-template` 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

Card 使用已弃用的 inline-template 属性接收其内容。

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

## 正确示例

省略 inline-template 属性，以通常方式传入相同内容。

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-inline-template": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20)
