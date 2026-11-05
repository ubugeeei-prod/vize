---
title: "css/no-important"
description: "不建议在 CSS 中使用 !important。"
---

# `css/no-important`

不建议在 CSS 中使用 !important。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

颜色声明通过 `!important` 覆盖普通层叠优先级。

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

## 正确示例

颜色来自自定义属性，没有使用 important 声明。

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
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
        "css/no-important": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 与 CSS 规则](../musea-and-css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14)
