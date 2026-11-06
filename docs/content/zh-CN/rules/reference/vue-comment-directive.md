---
title: "vue/comment-directive"
description: "支持 `<template>` 中的诊断控制注释。"
---

# `vue/comment-directive`

支持 `<template>` 中的诊断控制注释。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

只有启用 `reportUnusedDisableDirectives: true` 时，才会报告未使用的模板抑制注释。示例删除了无用的注释；实际抑制诊断的注释仍然有用。列出的规则名称只有在 Vize 执行这些规则时才参与使用情况判定。

## 错误示例

这个禁用所有规则的注释没有在后面的 `div` 上抑制任何诊断，因此会被未使用指令检查报告。

```vue annotate="remove:2"
<template>
  <!-- eslint-disable-next-line -->
  <div />
</template>
```

## 正确示例

删除无用的注释并保留原有元素，让这一行继续接受 lint 检查。

```vue
<template>
  <div />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/comment-directive": "error"
    },
    "reportUnusedDisableDirectives": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/comment_directive.rs#L33)
