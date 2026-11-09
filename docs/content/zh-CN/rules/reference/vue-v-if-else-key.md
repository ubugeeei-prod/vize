---
title: "vue/v-if-else-key"
description: "要求重复的条件组件具有 key。"
---

# `vue/v-if-else-key`

要求重复的条件组件具有 key。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

重复的条件 `Panel` 实例没有 key。

```vue annotate="remove:2"
<template>
  <section><Panel v-if="ready" /><Panel v-else /></section>
</template>
```

## 正确示例

每个条件组件都有稳定且唯一的 key。

```vue annotate="add:2"
<template>
  <section><Panel v-if="ready" key="ready" /><Panel v-else key="fallback" /></section>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-if-else-key": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules.rs#L162)
