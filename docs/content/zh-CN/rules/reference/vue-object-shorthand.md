---
title: "vue/object-shorthand"
description: "要求模板对象使用属性简写。"
---

# `vue/object-shorthand`

要求模板对象使用属性简写。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

对象同时重复了 `value` 键和值。

```vue annotate="remove:2"
<template>
  <Widget :options="{ value: value }" />
</template>
```

## 正确示例

属性简写保留同一绑定且不重复。

```vue annotate="add:2"
<template>
  <Widget :options="{ value }" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/object-shorthand": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules.rs#L138)
