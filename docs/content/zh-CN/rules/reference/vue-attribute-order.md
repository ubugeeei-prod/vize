---
title: "vue/attribute-order"
description: "要求属性保持一致的排列顺序。"
---

# `vue/attribute-order`

要求属性保持一致的排列顺序。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

事件处理器位于结构性 v-if 指令和普通 id 属性之前。

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

## 正确示例

按照规则的顺序，先写 v-if，再写 id，最后写事件处理器。

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
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
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36)
