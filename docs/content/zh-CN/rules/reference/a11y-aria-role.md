---
title: "a11y/aria-role"
description: "具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色。"
---

# `a11y/aria-role`

具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`datepicker` 不是此 section 上可识别的 ARIA 角色。

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

## 正确示例

section 使用可识别的 `dialog` 角色，以及描述日期选择的标签。

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L23)
