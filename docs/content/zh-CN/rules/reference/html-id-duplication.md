---
title: "html/id-duplication"
description: "禁止重复的元素 ID。"
---

# `html/id-duplication`

禁止重复的元素 ID。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

输入框和帮助段落都声明 `id="email"`，使标签目标不明确。

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

## 正确示例

输入框保留 `email`；帮助段落使用 `email-help`，aria-describedby 引用该独立 ID。

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/id-duplication": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [HTML 规则](../html.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36)
