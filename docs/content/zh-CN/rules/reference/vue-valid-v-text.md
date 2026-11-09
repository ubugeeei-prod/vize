---
title: "vue/valid-v-text"
description: "要求 `v-text` 指令有效。"
---

# `vue/valid-v-text`

要求 `v-text` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-text` 缺少文本表达式，或使用了不受支持的参数或修饰符。

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

## 正确示例

`v-text="msg"` 在语法上有效。独立的 `vue/no-v-text` 风格规则仍可能要求优先使用插值。

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-text": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L29)
