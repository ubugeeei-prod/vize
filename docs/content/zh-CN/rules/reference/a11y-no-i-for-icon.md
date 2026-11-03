---
title: "a11y/no-i-for-icon"
description: "禁止使用 &lt;i&gt; 元素表示图标。"
---

# `a11y/no-i-for-icon`

禁止使用 &lt;i&gt; 元素表示图标。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

图标通过 `i` 渲染，但其文本语义无法描述仅有图标的操作。

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

## 正确示例

装饰性 span 隐藏图标字形，独立的 `Delete item` 文本为按钮操作命名。

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35)
