---
title: "css/no-v-bind-performance"
description: "警告 CSS v-bind() 的性能开销。"
---

# `css/no-v-bind-performance`

警告 CSS v-bind() 的性能开销。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

样式表通过 SFC CSS 的 `v-bind()` 机制读取变化中的 `offset`。

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

## 正确示例

元素通过自身的样式绑定直接接收变化中的 transform。

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-v-bind-performance": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [CSS 规则](../css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20)
