---
title: "vue/no-restricted-block"
description: "禁止使用配置的块。"
---

# `vue/no-restricted-block`

禁止使用配置的块。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

SFC 包含被禁止的 i18n 块。

```vue annotate="remove:4"
<template>
  <p>ok</p>
</template>
<i18n lang="json">{}</i18n>
```

## 正确示例

SFC 只保留 template 块。

```vue
<template>
  <p>ok</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-block": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-block": [
        "i18n"
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/blocks.rs#L14)
