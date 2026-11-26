---
title: "vue/no-undefined-refs"
description: "禁止在模板中引用未定义的变量。"
---

# `vue/no-undefined-refs`

禁止在模板中引用未定义的变量。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

`globals` 列出由构建步骤为每个模板定义的名称，例如 `unplugin-auto-import` 的自动导入。在 Nuxt 项目中，`vize lint` 会从已生成的 `.nuxt/types/imports.d.ts` 读取自动导入并加入这些名称，因此无需填写；在 `nuxi prepare` 生成该文件之前，会输出提示并跳过此规则。名称按完全一致匹配。设置选项不会启用规则。后面的配置层会替换列表；空列表会清除继承的名称。

## 错误示例

模板读取 missing，但脚本只声明了 message。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

## 正确示例

插值读取已存在的 message 绑定。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-undefined-refs": "warn"
    },
    "ruleOptions": {
      "vue/no-undefined-refs": {
        "globals": [
          "formatCount"
        ]
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L13)
