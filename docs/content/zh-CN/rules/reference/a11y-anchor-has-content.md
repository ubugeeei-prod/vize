---
title: "a11y/anchor-has-content"
description: "要求锚点元素具有可访问的内容。"
---

# `a11y/anchor-has-content`

要求锚点元素具有可访问的内容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`/settings` 链接没有文本或其他命名内容，因此其目标没有可访问的描述。

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

## 正确示例

可见的 `Settings` 文本为同一目标链接提供内容。

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L19)
