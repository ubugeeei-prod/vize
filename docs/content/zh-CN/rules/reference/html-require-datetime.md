---
title: "html/require-datetime"
description: "要求 &lt;time&gt; 元素具有 datetime 属性。"
---

# `html/require-datetime`

要求 &lt;time&gt; 元素具有 datetime 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

time 元素包含人类可读的日期，却没有机器可读的 datetime 值。

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

## 正确示例

`datetime="2026-05-13"` 提供对应的机器可读日期。

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [HTML 规则](../html.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L36)
