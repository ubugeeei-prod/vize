---
title: "vue/v-on-event-hyphenation"
description: "要求组件上 v-on 中的自定义事件名称使用连字符。"
---

# `vue/v-on-event-hyphenation`

要求组件上 v-on 中的自定义事件名称使用连字符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

自定义组件监听器使用了 `@myEvent`，而不是连字符事件名称。

```vue annotate="remove:2,3,4"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
<my-component @update:modelValue="handler" />
</template>
```

## 正确示例

`@my-event` 符合要求的自定义事件写法。下方展示的原生元素监听器和动态事件参数不在此检查范围内。

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
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
        "vue/v-on-event-hyphenation": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L40)
