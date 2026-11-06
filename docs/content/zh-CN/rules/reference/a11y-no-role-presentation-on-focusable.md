---
title: "a11y/no-role-presentation-on-focusable"
description: "禁止在可获得焦点的元素上使用 role=\"presentation\" 或 role=\"none\"。"
---

# `a11y/no-role-presentation-on-focusable`

禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

可获得焦点的账单链接请求 role=presentation，与其交互式链接角色冲突；浏览器必须忽略该呈现角色请求。

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

## 正确示例

移除冲突的呈现角色请求，使用原生链接角色和账单目标。

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-role-presentation-on-focusable": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19)
