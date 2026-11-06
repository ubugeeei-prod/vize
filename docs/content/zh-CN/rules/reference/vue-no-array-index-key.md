---
title: "vue/no-array-index-key"
description: "禁止直接将 v-for 的索引变量用作 :key。"
---

# `vue/no-array-index-key`

禁止直接将 v-for 的索引变量用作 :key。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

列表使用当前索引作为 key，因此重新排列列表时，列表项的身份会发生变化。

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

## 正确示例

key 来自 item.id，即使位置改变，也能保持每个列表项的身份。

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-array-index-key": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32)
