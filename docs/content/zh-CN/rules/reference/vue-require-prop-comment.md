---
title: "vue/require-prop-comment"
description: "要求每个声明的 prop 都有 JSDoc 注释。"
---

# `vue/require-prop-comment`

要求每个声明的 prop 都有 JSDoc 注释。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

size 属性没有 JSDoc 注释。

```vue annotate="remove:2"
<script setup lang="ts">
const { size } = defineProps<{ size: string }>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## 正确示例

size 属性带有说明其含义的 JSDoc 注释。

```vue annotate="add:2,3,4,5"
<script setup lang="ts">
const { size } = defineProps<{
  /** The rendered size. */
  size: string;
}>();
</script>
<template>
  <p>{{ size }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-prop-comment": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_prop_comment.rs#L18)
