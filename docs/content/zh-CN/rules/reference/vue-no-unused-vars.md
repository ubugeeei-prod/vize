---
title: "vue/no-unused-vars"
description: "禁止 v-for 和 v-slot 指令中未使用的变量定义。"
---

# `vue/no-unused-vars`

禁止 v-for 和 v-slot 指令中未使用的变量定义。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

循环声明了未使用的 index，插槽也声明了未被引用的 foo。

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

## 正确示例

示例使用 index，或将其命名为 _index 以标记有意不使用；插槽则渲染 data。这里的索引 key 只展示变量使用，并不建议用它保持列表项身份稳定。

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-vars": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L47)
