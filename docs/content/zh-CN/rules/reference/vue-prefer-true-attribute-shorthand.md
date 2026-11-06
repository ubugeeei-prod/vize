---
title: "vue/prefer-true-attribute-shorthand"
description: "优先使用绑定到 `true` 的布尔属性简写。"
---

# `vue/prefer-true-attribute-shorthand`

优先使用绑定到 `true` 的布尔属性简写。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

原生布尔属性 disabled 绑定了常量 true。

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

## 正确示例

原生属性使用布尔简写。false 绑定和组件 props 保留显式值。

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/prefer-true-attribute-shorthand": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38)
