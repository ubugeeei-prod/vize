---
title: "vue/permitted-contents"
description: "要求符合 HTML 内容模型规则。"
---

# `vue/permitted-contents`

要求符合 HTML 内容模型规则。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

示例在 p 中放置块级内容、省略表格主体、嵌套交互控件，或将 div 直接放在 ul 内。

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

## 正确示例

示例使用段落内的行内内容、显式 tbody 和 li 子节点。自定义 MyItem 不被视为已知的原生 ul 子节点。

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/permitted-contents": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56)
