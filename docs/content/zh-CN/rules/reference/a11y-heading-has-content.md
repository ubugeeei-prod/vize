---
title: "a11y/heading-has-content"
description: "要求标题元素具有可访问的内容。"
---

# `a11y/heading-has-content`

要求标题元素具有可访问的内容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`h2` 提供了标题层级，却没有标题内容。

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

## 正确示例

`Billing settings` 为原有二级标题提供内容。

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L18)
