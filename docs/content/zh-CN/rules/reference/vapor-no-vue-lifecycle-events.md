---
title: "vapor/no-vue-lifecycle-events"
description: "禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）。"
---

# `vapor/no-vue-lifecycle-events`

禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

输入框使用了 @vue:mounted 模板生命周期事件。

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

## 正确示例

onMounted 通过受支持的脚本生命周期钩子访问具名模板引用，并聚焦输入框。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vapor 规则](../vapor.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34)
