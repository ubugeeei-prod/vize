---
title: "a11y/no-static-element-interactions"
description: "禁止在静态元素上设置事件处理器。"
---

# `a11y/no-static-element-interactions`

禁止在静态元素上设置事件处理器。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

静态 section 接收 Enter 键操作，却没有交互角色。

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

## 正确示例

原生按钮以适当的交互元素承载相同操作。

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-static-element-interactions": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L37)
