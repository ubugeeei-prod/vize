---
title: "a11y/mouse-events-have-key-events"
description: "要求鼠标事件配有 focus/blur 事件。"
---

# `a11y/mouse-events-have-key-events`

要求鼠标事件配有 focus/blur 事件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

预览可见性只通过鼠标进入和离开处理器改变。

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

## 正确示例

相同的预览操作也在 focus 和 blur 时执行，并且按钮可以获得键盘焦点。

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
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
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30)
