---
title: "vue/no-restricted-component-names"
description: "禁止使用配置的组件名。"
---

# `vue/no-restricted-component-names`

禁止使用配置的组件名。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件名是被禁止的 RestrictedShowcase。

```vue annotate="remove:2"
<script setup lang="ts">
defineOptions({ name: "RestrictedShowcase" });
</script>
<template>
  <p>ok</p>
</template>
```

## 正确示例

组件名是允许的 Widget。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: "Widget" });
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
      "vue/no-restricted-component-names": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-component-names": [
        "RestrictedShowcase"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/component_names.rs#L12)
