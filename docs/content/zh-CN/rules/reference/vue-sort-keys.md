---
title: "vue/sort-keys"
description: "按配置顺序排列组件选项中的键。"
---

# `vue/sort-keys`

按配置顺序排列组件选项中的键。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

嵌套的方法键未按升序排列。

```vue annotate="remove:2"
<script>
export default { methods: { zebra() {}, alpha() {} } }
</script>
<template />
```

## 正确示例

嵌套的方法键遵循配置顺序。

```vue annotate="add:2"
<script>
export default { methods: { alpha() {}, zebra() {} } }
</script>
<template />
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/sort-keys": "warn"
    },
    "ruleOptions": {
      "vue/sort-keys": {
        "order": "asc",
        "caseSensitive": true,
        "natural": false
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_sfc/options.rs#L23)
