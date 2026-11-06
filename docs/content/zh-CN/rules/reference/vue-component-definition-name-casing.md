---
title: "vue/component-definition-name-casing"
description: "要求组件定义名称使用 PascalCase 或 kebab-case。"
---

# `vue/component-definition-name-casing`

要求组件定义名称使用 PascalCase 或 kebab-case。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

检查的是组件文件名。接受 PascalCase 和 kebab-case；混合大小写会被报告。

## 错误示例

文件名 myComponent.vue 以小写字母开头，却在内部使用大写字母，既不是 PascalCase，也不是 kebab-case。

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

## 正确示例

将文件重命名为 MyComponent.vue 即采用 PascalCase；模板内容无需更改。

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/component-definition-name-casing": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36)
