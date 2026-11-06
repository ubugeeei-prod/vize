---
title: "vue/no-inline-style"
description: "不建议使用内联 style 属性。"
---

# `vue/no-inline-style`

不建议使用内联 style 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

静态 style 属性将颜色声明直接嵌入元素。

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

## 正确示例

类名表达固定颜色；依赖 ratio 的宽度仍使用动态样式绑定，不属于静态属性检查的范围。

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-inline-style": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33)
