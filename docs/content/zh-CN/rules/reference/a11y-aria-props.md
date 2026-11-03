---
title: "a11y/aria-props"
description: "禁止无效的 ARIA 属性。"
---

# `a11y/aria-props`

禁止无效的 ARIA 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`aria-lable` 拼写错误，不是受支持的 ARIA 属性。

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

## 正确示例

受支持的 `aria-label` 属性提供按钮名称。

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18)
