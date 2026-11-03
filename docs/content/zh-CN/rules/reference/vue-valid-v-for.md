---
title: "vue/valid-v-for"
description: "要求 `v-for` 指令有效。"
---

# `vue/valid-v-for`

要求 `v-for` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

循环省略了迭代表达式，或添加了不受支持的 `.stop` 修饰符。

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

## 正确示例

使用 `item in items` 或 `(item, index) of items`，提供完整的迭代表达式及示例中的 key。

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
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
        "vue/valid-v-for": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L39)
