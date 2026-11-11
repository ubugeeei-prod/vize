---
title: "vue/no-restricted-props"
description: "禁止使用配置的 props。"
---

# `vue/no-restricted-props`

禁止使用配置的 props。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件声明了被禁止的 third prop。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ third?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## 正确示例

组件声明允许的 first prop。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ first?: number }>();
</script>
<template>
  <p>ok</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-props": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-props": [
        "third"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_props.rs#L12)
