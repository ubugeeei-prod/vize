---
title: "css/prefer-slotted"
description: "建议使用 ::v-slotted() 为插槽内容设置样式。"
---

# `css/prefer-slotted`

建议使用 ::v-slotted() 为插槽内容设置样式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

scoped 样式表选择的是 `slot` 出口，而不是通过插槽传入的元素。

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

## 正确示例

`:slotted(.label)` 通过作用域插槽选择器选中传入的 label 元素。

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
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
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 与 CSS 规则](../musea-and-css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34)
