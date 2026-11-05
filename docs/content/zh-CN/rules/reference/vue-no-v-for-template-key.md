---
title: "vue/no-v-for-template-key"
description: "为兼容 Vue 2，要求将 `template v-for` 的 key 放在实际子元素上。"
---

# `vue/no-v-for-template-key`

为兼容 Vue 2，要求将 `template v-for` 的 key 放在实际子元素上。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

这是一条按需启用的 Vue 2 兼容性规则。Vue 2 要求将 key 放在实际子元素上；Vue 3 则要求放在 `<template v-for>` 上。要检查 Vue 3 的约定，请使用 `vue/no-v-for-template-key-on-child`，不要同时启用这两种约定。

## 错误示例

在 Vue 2 约定中，`<template v-for>` 不会渲染实际元素，其上的 key 会被忽略。

```vue annotate="remove:1"
<template><ul><template v-for="item in items" :key="item.id"><li>{{ item.label }}</li></template></ul></template>
```

## 正确示例

按照这里检查的 Vue 2 约定，将列表项的 key 放在重复渲染的实际子元素 `li` 上。

```vue annotate="add:1"
<template><ul><template v-for="item in items"><li :key="item.id">{{ item.label }}</li></template></ul></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key.rs#L33)
