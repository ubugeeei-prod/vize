---
title: "vue/valid-attribute-name"
description: "要求属性名称有效。"
---

# `vue/valid-attribute-name`

要求属性名称有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

错误示例的诊断：`parser/template`

格式错误的属性写法会先由 parser/template 诊断，之后该防御性规则才能读取属性。因此 Bad 报告的是 parser/template，并不保证另有独立的 vue/valid-attribute-name 问题。

## 错误示例

`my"attr` 内的引号使属性名称格式错误。此示例产生解析器的 `parser/template` 诊断，并不保证另有独立的规则诊断。

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

## 正确示例

`my-attr` 是格式正确的属性名称，因此模板解析器可以读取该属性及其值。

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
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
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27)
