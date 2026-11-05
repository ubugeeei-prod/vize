---
title: "a11y/img-alt"
description: "要求图片提供 alt 属性，以支持无障碍访问。"
---

# `a11y/img-alt`

要求图片提供 alt 属性，以支持无障碍访问。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

头像图片缺少 `alt` 属性。

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

## 正确示例

`alt="User avatar"` 为头像提供文本替代。

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L17)
