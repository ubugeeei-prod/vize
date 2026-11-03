---
title: "vue/valid-v-is"
description: "要求 `v-is` 指令有效。"
---

# `vue/valid-v-is`

要求 `v-is` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-is` 带有参数或修饰符、缺少值，或用在组件上而不是原生 HTML 元素上。

```vue annotate="remove:2,3,4,5"
<template>
<tr v-is:arg="'MyRow'"></tr>
<tr v-is.mod="'MyRow'"></tr>
<tr v-is></tr>
<MyComponent v-is="'MyRow'" />
</template>
```

## 正确示例

在 `<tr>` 等原生元素上使用带值的 `v-is`，指定要在该位置渲染的组件。

```vue annotate="add:2"
<template>
<tr v-is="'MyRow'"></tr>
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
        "vue/valid-v-is": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_is.rs#L30)
