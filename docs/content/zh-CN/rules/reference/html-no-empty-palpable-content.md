---
title: "html/no-empty-palpable-content"
description: "禁止预期具有可见内容的空元素。"
---

# `html/no-empty-palpable-content`

禁止预期具有可见内容的空元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

段落、列表项和表格单元格都没有可感知的内容。

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

## 正确示例

文本填充段落，插值提供列表项内容，aria-label 则显式命名原本为空的单元格。

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-empty-palpable-content": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [HTML 规则](../html.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32)
