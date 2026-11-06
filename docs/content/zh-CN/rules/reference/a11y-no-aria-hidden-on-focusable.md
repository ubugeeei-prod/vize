---
title: "a11y/no-aria-hidden-on-focusable"
description: "禁止在可获得焦点的元素上使用 aria-hidden=\"true\"。"
---

# `a11y/no-aria-hidden-on-focusable`

禁止在可获得焦点的元素上使用 aria-hidden="true"。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

可获得焦点的 Close 按钮使用 `aria-hidden="true"`，从无障碍树中隐藏。

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

## 正确示例

按钮继续对辅助技术可见，并获得 `Close` 标签，不再被隐藏。

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-aria-hidden-on-focusable": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19)
