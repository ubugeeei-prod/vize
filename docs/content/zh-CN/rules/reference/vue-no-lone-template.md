---
title: "vue/no-lone-template"
description: "禁止不必要的 `<template>` 元素。"
---

# `vue/no-lone-template`

禁止不必要的 `<template>` 元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

内层 template 没有指令或插槽作用，因此不具备结构上的用途。

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

## 正确示例

移除不必要的包装层后，段落直接位于 div 内。

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
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
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32)
