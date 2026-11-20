---
title: "css/require-font-display"
description: "要求 @font-face 规则具有 font-display。"
---

# `css/require-font-display`

要求 @font-face 规则具有 font-display。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：CSS、SCSS、Sass、Less 和 Stylus 的 style 块，按原样读取（不运行预处理器）。同一个 Stylus 块内请统一使用花括号语法或缩进语法；混合语法会按花括号 Stylus 读取。

## 错误示例

font-face 声明定义了字体来源，却省略了 font-display 策略。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

## 正确示例

`font-display: swap` 显式选择先显示后备字体、再切换到目标字体的策略。

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/require-font-display": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [CSS 规则](../css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L15)
