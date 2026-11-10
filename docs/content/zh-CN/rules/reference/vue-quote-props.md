---
title: "vue/quote-props"
description: "禁止不必要的带引号对象属性。"
---

# `vue/quote-props`

禁止不必要的带引号对象属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

可作为标识符的 `name` 键被多余地加上引号。

```vue annotate="remove:2"
<template>
  <Widget :options='{ "name": value }' />
</template>
```

## 正确示例

属性使用无引号的标识符形式。

```vue annotate="add:2"
<template>
  <Widget :options="{ name: value }" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/quote-props": "warn"
    },
    "ruleOptions": {
      "vue/quote-props": "as-needed"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/quote_props.rs#L10)
