---
title: "a11y/interactive-supports-focus"
description: "要求具有交互角色的元素可获得焦点。"
---

# `a11y/interactive-supports-focus`

要求具有交互角色的元素可获得焦点。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

为 `span` 设置按钮角色和点击处理器，并不会让它能够通过键盘获得焦点。

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

## 正确示例

原生按钮可以获得焦点，并保留同一 `open` 操作。

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L39)
