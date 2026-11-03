---
title: "vue/no-v-for-template-key-on-child"
description: "禁止在 `<template v-for>` 的子节点上使用 `key`。"
---

# `vue/no-v-for-template-key-on-child`

禁止在 `<template v-for>` 的子节点上使用 `key`。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

子段落具有 key，但 template 迭代自身没有 key。

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

## 正确示例

key 移到 template v-for 上，用来标识整个重复片段。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30)
