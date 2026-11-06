---
title: "css/no-display-none"
description: "建议使用 v-show 代替 display: none。"
---

# `css/no-display-none`

建议使用 v-show 代替 display: none。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

`.message` 声明通过 CSS 隐藏当前段落，没有使用模板可见性条件。

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

## 正确示例

`v-show="isSaved"` 在当前段落上显式表达可见性条件，并移除 `display: none`。

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-display-none": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [CSS 规则](../css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27)
