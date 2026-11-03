---
title: "vue/no-unsafe-url"
description: "警告可能不安全的 URL 绑定。"
---

# `vue/no-unsafe-url`

警告可能不安全的 URL 绑定。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

锚点的目标地址以可执行的 javascript: 协议开头。

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

## 正确示例

锚点使用普通的本地导航目标 /next。

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
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
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55)
