---
title: "vue/no-template-key"
description: "禁止在 `<template>` 上使用 `key` 属性。"
---

# `vue/no-template-key`

禁止在 `<template>` 上使用 `key` 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

非循环的 template 包装层具有 key，但它并不是需要 key 的迭代边界。

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

## 正确示例

key 属于 template v-for 迭代，用来标识每个重复的片段。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-key": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L29)
