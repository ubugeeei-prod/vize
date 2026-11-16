---
title: "vue/valid-v-pre"
description: "要求 `v-pre` 指令有效。"
---

# `vue/valid-v-pre`

要求 `v-pre` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-pre` 设置了参数、修饰符或值，但此指令是跳过编译的无值标记。

```vue annotate="remove:2,3,4"
<template>
<div v-pre:arg></div>
<div v-pre.mod></div>
<div v-pre="foo"></div>
</template>
```

## 正确示例

不带值的 `v-pre` 让子树跳过编译，其中的插值按原样显示。

```vue annotate="add:2"
<template>
<div v-pre>{{ raw }}</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-pre": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_pre.rs#L32)
