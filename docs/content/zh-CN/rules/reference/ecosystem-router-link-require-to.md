---
title: "ecosystem/router-link-require-to"
description: "要求 RouterLink 和 NuxtLink 组件具有 `to` 目标。"
---

# `ecosystem/router-link-require-to`

要求 RouterLink 和 NuxtLink 组件具有 `to` 目标。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `ecosystem` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

SFC 的唯一根链接可能从父组件属性继承目标。此示例使用嵌套链接，必须显式提供目标。

## 错误示例

嵌套 RouterLink 没有 `to` 目标，不能依赖根节点的属性透传。

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

## 正确示例

`to="/settings"` 显式提供嵌套链接目标。

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/router-link-require-to": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14)
