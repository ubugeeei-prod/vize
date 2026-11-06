---
title: "css/no-utility-classes"
description: "警告在组件样式中实现工具类。"
---

# `css/no-utility-classes`

警告在组件样式中实现工具类。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

编写的选择器采用 `.flex`、`.mt-4` 和 `.text-center` 等工具类形式的名称。

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

## 正确示例

组件专用的 `.my-component` 选择器以一个语义名称组织组件样式。

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-utility-classes": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [CSS 规则](../css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37)
