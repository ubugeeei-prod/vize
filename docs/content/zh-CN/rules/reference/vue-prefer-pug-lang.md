---
title: "vue/prefer-pug-lang"
description: "优先使用 lang=\"pug\"，而不是旧版 jade 1.x 模板语言。"
---

# `vue/prefer-pug-lang`

优先使用 lang="pug"，而不是旧版 jade 1.x 模板语言。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板指定 jade，Vue 会用旧版 jade 1.x 包渲染它，并且使用了 jade 特有的带短横线的 each。

```vue annotate="remove:1,3"
<template lang="jade">
ul
  - each item in [1, 2]
    li= item
</template>
```

## 正确示例

模板指定 pug，并用 pug 3 的 each 关键字编写循环。

```vue annotate="add:1,3"
<template lang="pug">
ul
  each item in [1, 2]
    li= item
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-pug-lang": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prefer_pug_lang.rs#L42)
