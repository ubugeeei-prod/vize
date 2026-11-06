---
title: "a11y/tabindex-no-positive"
description: "禁止正数 tabindex 值。"
---

# `a11y/tabindex-no-positive`

禁止正数 tabindex 值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

正数 tabindex 3 建立了位于普通控件之前的自定义焦点顺序。

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

## 正确示例

按钮使用原生焦点顺序，不设置正数 tabindex。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/tabindex-no-positive": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16)
