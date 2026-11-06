---
title: "vue/no-empty-component-block"
description: "禁止空的 SFC 块。"
---

# `vue/no-empty-component-block`

禁止空的 SFC 块。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

template、script 和 style 块都没有实质内容。

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

## 正确示例

每个保留的块都包含实际的标记、脚本声明或样式声明。

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
  <div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-component-block": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42)
