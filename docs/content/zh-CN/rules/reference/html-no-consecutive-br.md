---
title: "html/no-consecutive-br"
description: "禁止连续的 &lt;br&gt; 元素。"
---

# `html/no-consecutive-br`

禁止连续的 &lt;br&gt; 元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

两个连续换行元素在同一段落内为内容块制造间距。

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

## 正确示例

使用独立段落表达两个内容块，无需重复换行元素。

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "html/no-consecutive-br": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [HTML 规则](../html.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30)
