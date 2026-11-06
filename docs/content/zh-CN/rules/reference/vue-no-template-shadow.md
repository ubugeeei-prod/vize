---
title: "vue/no-template-shadow"
description: "禁止使用遮蔽外层作用域变量的变量名称。"
---

# `vue/no-template-shadow`

禁止使用遮蔽外层作用域变量的变量名称。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

此检查比较模板内的 `v-for`、`v-slot`（包括 `#name`）和旧式作用域插槽绑定。它报告遮蔽先前绑定的标识符，也包括按属性顺序比较的同一元素上的声明。兄弟元素具有独立的作用域。属性键和默认值中的引用不属于声明，`v-pre` 会使其子树不参与检查。

脚本和组件属性的名称不在当前检查范围内：单个模板绑定不会仅仅因为与脚本绑定同名就产生警告。诊断范围覆盖源代码中实际编写的标识符，包括转义写法和 HTML 实体。

## 错误示例

内层 v-for 再次声明 item，在嵌套循环中遮蔽了外层 item 绑定。

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

## 正确示例

内层循环声明 child，外层行仍使用 item，内层行则使用 child。

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-template-shadow": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34)
