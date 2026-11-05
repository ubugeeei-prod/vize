---
title: "a11y/no-redundant-roles"
description: "禁止多余的 ARIA 角色。"
---

# `a11y/no-redundant-roles`

禁止多余的 ARIA 角色。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

原生按钮已具有按钮角色，因此 `role="button"` 重复了其隐式语义。

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

## 正确示例

移除重复角色，保留 HTML 提供的按钮语义。

```vue annotate="add:2"
<template>
  <button>Save</button>
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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31)
