---
title: "vue/no-unsupported-features"
description: "禁止在指定的 Vue.js 版本上使用尚未支持的语法。"
---

# `vue/no-unsupported-features`

禁止在指定的 Vue.js 版本上使用尚未支持的语法。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

defineModel 需要比配置版本更新的 Vue。

```vue annotate="remove:2,5"
<script setup lang="ts">
const model = defineModel<string>();
</script>
<template>
  <p>{{ model }}</p>
</template>
```

## 正确示例

组件不使用受版本限制的宏。

```vue annotate="add:2,5"
<script setup lang="ts">
const label = "ok";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unsupported-features": "warn"
    },
    "ruleOptions": {
      "vue/no-unsupported-features": {
        "version": "^3.3.0"
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/unsupported_features.rs#L19)
