---
title: "a11y/use-list"
description: "建议为类似项目符号的文本使用列表元素。"
---

# `a11y/use-list`

建议为类似项目符号的文本使用列表元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

任务项使用单独的段落和手写短横线标记，没有使用列表元素。

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

## 正确示例

无序列表及列表项以列表语义表达相同任务。

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36)
