---
title: "a11y/no-autofocus"
description: "禁止使用 autofocus 属性。"
---

# `a11y/no-autofocus`

禁止使用 autofocus 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

输入框在出现时请求自动获得焦点。

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

## 正确示例

移除 `autofocus`，避免该自动焦点请求，同时保留查询输入框。

```vue annotate="add:2"
<template>
  <input name="query" />
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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19)
