---
title: "a11y/aria-unsupported-elements"
description: "禁止在不支持 ARIA 的元素上使用 ARIA 属性。"
---

# `a11y/aria-unsupported-elements`

禁止在不支持 ARIA 的元素上使用 ARIA 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

元数据元素带有 `aria-hidden`，但 `meta` 不支持 ARIA 属性。

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

## 正确示例

移除 ARIA 属性，保留字符集声明。

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/aria-unsupported-elements": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L21)
