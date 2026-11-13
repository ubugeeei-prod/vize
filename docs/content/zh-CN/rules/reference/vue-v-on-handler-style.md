---
title: "vue/v-on-handler-style"
description: "要求 v-on 处理器写为方法引用。"
---

# `vue/v-on-handler-style`

要求 v-on 处理器写为方法引用。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

处理器使用内联函数、修改操作或多条语句，而不是可复用的方法引用。

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

## 正确示例

使用处理器引用。将内联逻辑移到具名方法中，使模板保留稳定且可复用的处理器。

```vue annotate="add:2,3"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/v-on-handler-style": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L38)
