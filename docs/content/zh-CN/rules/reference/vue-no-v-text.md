---
title: "vue/no-v-text"
description: "禁止 v-text 指令，优先使用双花括号插值。"
---

# `vue/no-v-text`

禁止 v-text 指令，优先使用双花括号插值。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

div 的内容通过 v-text 指令提供。

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

## 正确示例

双花括号插值直接在元素内容中表达相同的文本绑定。

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-v-text": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L32)
