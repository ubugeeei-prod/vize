---
title: "vue/require-scoped-style"
description: "要求 style 标签具有 scoped 属性。"
---

# `vue/require-scoped-style`

要求 style 标签具有 scoped 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`.button` 样式没有作用域限制，可能影响组件外部匹配的元素。

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

## 正确示例

添加 `scoped`，为同样的选择器和声明应用 Vue 的组件作用域。

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
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
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49)
