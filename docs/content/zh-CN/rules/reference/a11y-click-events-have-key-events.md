---
title: "a11y/click-events-have-key-events"
description: "要求点击事件配有键盘事件处理器。"
---

# `a11y/click-events-have-key-events`

要求点击事件配有键盘事件处理器。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

检查没有交互角色的非交互式元素。原生按钮和具有交互式 ARIA 角色的元素不在此规则诊断范围内。

## 错误示例

非交互式 `div` 有点击处理器，却没有键盘事件处理。

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

## 正确示例

原生 `button` 为同一 `activate` 处理器提供键盘激活方式。

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
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
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L22)
