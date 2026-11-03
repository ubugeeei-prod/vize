---
title: "vue/no-unsandboxed-iframe"
description: "要求 iframe 元素具有 sandbox 属性。"
---

# `vue/no-unsandboxed-iframe`

要求 iframe 元素具有 sandbox 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

嵌入的框架没有使用 sandbox 属性限制其能力。

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

## 正确示例

sandbox 施加限制；需要脚本能力时，allow-scripts 显式允许这一项能力。

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
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
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32)
