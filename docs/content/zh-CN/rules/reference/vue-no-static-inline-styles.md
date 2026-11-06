---
title: "vue/no-static-inline-styles"
description: "禁止静态内联 style 属性。"
---

# `vue/no-static-inline-styles`

禁止静态内联 style 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

段落在 style 属性中包含固定的颜色声明。

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

## 正确示例

notice 类和 scoped 样式表将固定颜色放在模板属性之外。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-static-inline-styles": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15)
