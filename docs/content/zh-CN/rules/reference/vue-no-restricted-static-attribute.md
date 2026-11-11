---
title: "vue/no-restricted-static-attribute"
description: "禁止使用配置的静态属性。"
---

# `vue/no-restricted-static-attribute`

禁止使用配置的静态属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

按钮带有被禁止的 data-legacy 属性。

```vue annotate="remove:2"
<template>
  <button type="button" data-legacy="x">x</button>
</template>
```

## 正确示例

按钮只保留 type。

```vue annotate="add:2"
<template>
  <button type="button">x</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-static-attribute": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-static-attribute": [
        "data-legacy"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_static_attribute.rs#L13)
