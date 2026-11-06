---
title: "css/prefer-nested-selectors"
description: "建议为后代选择器使用 CSS 嵌套。"
---

# `css/prefer-nested-selectors`

建议为后代选择器使用 CSS 嵌套。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

`.card .title` 后代选择器在平铺规则中重复了父选择器。

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

## 正确示例

`.title` 规则嵌套在 `.card` 内，将父子样式关系放在一起。

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [CSS 规则](../css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14)
