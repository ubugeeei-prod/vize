---
title: "a11y/label-has-for"
description: "要求标签具有相关联的表单控件。"
---

# `a11y/label-has-for`

要求标签具有相关联的表单控件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

独立的 label 既没有通过 `for` 关联输入框，也没有包住输入框。

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

## 正确示例

`for="email"` 与输入框 ID 一致，显式关联两个元素。

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L31)
