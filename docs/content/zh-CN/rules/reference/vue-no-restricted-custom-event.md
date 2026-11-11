---
title: "vue/no-restricted-custom-event"
description: "禁止使用配置的自定义事件。"
---

# `vue/no-restricted-custom-event`

禁止使用配置的自定义事件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板通过 $emit 发出被禁止的 legacyEvent。

```vue annotate="remove:2"
<template>
  <button type="button" @click="$emit('legacyEvent')">x</button>
</template>
```

## 正确示例

模板发出允许的 save 事件。

```vue annotate="add:2"
<template>
  <button type="button" @click="$emit('save')">x</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-custom-event": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-custom-event": [
        "legacyEvent"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_custom_event.rs#L19)
