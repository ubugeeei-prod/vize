---
title: "vue/no-deprecated-v-is"
description: "禁止使用已废弃的 `v-is` 指令。"
---

# `vue/no-deprecated-v-is`

禁止使用已废弃的 `v-is` 指令。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`v-is` 已在 Vue 3.1 中废弃；示例仍用它切换渲染的组件。

```vue annotate="remove:2"
<template>
<tr v-is="'MyRow'"></tr>
</template>
```

## 正确示例

改用带 `vue:` 前缀的 `is` 属性，或使用 `<component :is>` 选择组件。

```vue annotate="add:2,3"
<template>
<tr is="vue:MyRow"></tr>
<component :is="MyRow" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-v-is": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_is.rs#L27)
