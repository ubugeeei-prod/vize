---
title: "vue/no-preprocessor-lang"
description: "不建议使用 CSS 预处理器，优先使用现代 CSS。"
---

# `vue/no-preprocessor-lang`

不建议使用 CSS 预处理器，优先使用现代 CSS。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

style 块通过 lang 选择 SCSS。这描述了不使用预处理器的预期规范；当前 SFC 路径不会报告此规则。

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

## 正确示例

相同的 CSS 声明省略了预处理器 lang。这修正了规范上的问题，但目前不会产生可执行的 Bad/Good 诊断差异。

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L11)
