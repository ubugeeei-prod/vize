---
title: "a11y/alt-text"
description: "要求媒体元素提供替代文本。"
---

# `a11y/alt-text`

要求媒体元素提供替代文本。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

图片提交控件只提供图片 URL，没有描述操作的 `alt` 文本。

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

## 正确示例

`alt="Submit search"` 为图片控件提供描述提交搜索操作的无障碍名称。

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L37)
