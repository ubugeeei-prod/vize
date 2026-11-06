---
title: "ecosystem/nuxt-prefer-nuxt-link"
description: "应用内部链接优先使用 NuxtLink。"
---

# `ecosystem/nuxt-prefer-nuxt-link`

应用内部链接优先使用 NuxtLink。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

Nuxt 应用中的内部设置目标使用了普通锚点。

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

## 正确示例

NuxtLink 通过 Nuxt 路由处理相同的内部目标。

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "ecosystem/nuxt-prefer-nuxt-link": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [生态系统与 Nuxt 规则](../ecosystem.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14)
