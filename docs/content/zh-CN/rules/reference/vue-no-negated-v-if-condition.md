---
title: "vue/no-negated-v-if-condition"
description: "禁止在含有 v-else 的条件链中使用否定的 v-if 条件。"
---

# `vue/no-negated-v-if-condition`

禁止在含有 v-else 的条件链中使用否定的 v-if 条件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

成对的 v-if 和 v-else 分支以否定条件开头。

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

## 正确示例

先使用肯定条件 ok；反转条件时，将原来的相反分支放在前面。单独使用否定的 v-if 和 !== 比较仍被允许。

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-negated-v-if-condition": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L41)
