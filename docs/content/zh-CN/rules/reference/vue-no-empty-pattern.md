---
title: "vue/no-empty-pattern"
description: "禁止模板表达式中的空解构模式。"
---

# `vue/no-empty-pattern`

禁止模板表达式中的空解构模式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

回调解构了一个空对象模式。

```vue annotate="remove:2"
<template>
  <p>{{ items.map(({}) => 1) }}</p>
</template>
```

## 正确示例

回调明确命名其使用的属性。

```vue annotate="add:2"
<template>
  <p>{{ items.map(({ id }) => id) }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-empty-pattern": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L89)
