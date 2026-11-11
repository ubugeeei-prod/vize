---
title: "vue/prefer-v-model"
description: "用 v-model 代替匹配的 bind 与 update 监听器。"
---

# `vue/prefer-v-model`

用 v-model 代替匹配的 bind 与 update 监听器。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

bind 和 update 监听器手动同步同一个值。

```vue annotate="remove:2"
<template>
  <TextField :model-value="value" @update:model-value="value = $event" />
</template>
```

## 正确示例

`v-model` 一次表达相同的双向组件约定。

```vue annotate="add:2"
<template>
  <TextField v-model="value" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-v-model": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/template_rules/prefer_v_model.rs#L19)
