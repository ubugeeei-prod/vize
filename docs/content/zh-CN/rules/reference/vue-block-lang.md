---
title: "vue/block-lang"
description: "将 SFC 块的 lang 限制为配置的语言。"
---

# `vue/block-lang`

将 SFC 块的 lang 限制为配置的语言。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

script 块缺少配置的 lang 属性。

```vue annotate="remove:1"
<script setup>
const label = "a";
</script>
<template>
  <p>{{ label }}</p>
</template>
```

## 正确示例

script 块把 lang 设为配置的语言。

```vue annotate="add:1"
<script setup lang="ts">
const label = "a";
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
      "vue/block-lang": "warn"
    },
    "ruleOptions": {
      "vue/block-lang": {
        "script": {
          "lang": "ts",
          "allowNoLang": false
        }
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/block_lang.rs#L12)
