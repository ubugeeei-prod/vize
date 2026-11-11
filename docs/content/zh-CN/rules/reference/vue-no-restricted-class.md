---
title: "vue/no-restricted-class"
description: "禁止使用配置的 class。"
---

# `vue/no-restricted-class`

禁止使用配置的 class。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

元素使用了被禁止的 legacy class。

```vue annotate="remove:2"
<template>
  <div class="legacy">x</div>
</template>
```

## 正确示例

元素使用允许的 class。

```vue annotate="add:2"
<template>
  <div class="current">x</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-class": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-class": [
        "legacy"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/restricted_class.rs#L13)
