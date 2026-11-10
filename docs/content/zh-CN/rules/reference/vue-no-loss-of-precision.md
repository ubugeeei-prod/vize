---
title: "vue/no-loss-of-precision"
description: "禁止会丢失精度的数字字面量。"
---

# `vue/no-loss-of-precision`

禁止会丢失精度的数字字面量。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

该整数无法用 JavaScript number 精确表示。

```vue annotate="remove:2"
<template>
  <p>{{ 9007199254740993 }}</p>
</template>
```

## 正确示例

预先计算的精确绑定避免不精确的模板字面量。

```vue annotate="add:2"
<template>
  <p>{{ preciseValue }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-loss-of-precision": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_loss_of_precision.rs#L12)
