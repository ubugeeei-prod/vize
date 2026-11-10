---
title: "vue/valid-v-memo"
description: "要求 `v-memo` 指令有效。"
---

# `vue/valid-v-memo`

要求 `v-memo` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

不带值的 `v-memo` 没有为 Vue 提供依赖表达式，无法判断何时复用子树。

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

## 正确示例

`v-memo="[valueA, valueB]"` 提供用于记忆化的依赖数组。

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/valid-v-memo": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L32)
