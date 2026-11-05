---
title: "a11y/role-has-required-aria-props"
description: "要求 ARIA 角色具有必需属性。"
---

# `a11y/role-has-required-aria-props`

要求 ARIA 角色具有必需属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

checkbox 角色省略了表达复选框状态的 `aria-checked`。

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

## 正确示例

`aria-checked="false"` 提供 checkbox 角色所需的状态。

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30)
