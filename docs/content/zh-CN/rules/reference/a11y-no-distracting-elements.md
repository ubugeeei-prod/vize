---
title: "a11y/no-distracting-elements"
description: "禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素。"
---

# `a11y/no-distracting-elements`

禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`marquee` 元素引入了自动移动的文本。

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

## 正确示例

段落显示同样的优惠信息，不使用分散注意力的 marquee 元素。

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-distracting-elements": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16)
