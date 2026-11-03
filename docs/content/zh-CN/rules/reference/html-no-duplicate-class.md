---
title: "html/no-duplicate-class"
description: "禁止静态 class 属性中重复的类名。"
---

# `html/no-duplicate-class`

禁止静态 class 属性中重复的类名。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

静态类名列表重复了 `btn`。

```vue annotate="remove:2"
<template>
  <div class="btn btn primary">click</div>
</template>
```

## 正确示例

类名列表保留一个 `btn` 及不同的 `primary`。

```vue annotate="add:2"
<template>
  <div class="btn primary">click</div>
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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [HTML 规则](../html.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32)
