---
title: "a11y/label-has-for"
description: "要求标签具有相关联的表单控件。"
---

# `a11y/label-has-for`

要求标签具有相关联的表单控件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

列出会渲染原生表单控件、因而满足 label 嵌套条件的组件名。PascalCase 与 kebab-case 写法等价。选项不会启用规则；后续配置层会替换整个列表，[] 会清除继承的名称。

## 错误示例

独立的 label 既没有通过 `for` 关联输入框，也没有包住输入框。

```vue annotate="remove:2"
<template>
<label>Name</label>
</template>
```

## 正确示例

嵌套的原生 input 与包裹它的 label 隐式关联。

```vue annotate="add:2"
<template>
<label>Name <input v-model="name" /></label>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/label-has-for": "warn"
    },
    "ruleOptions": {
      "a11y/label-has-for": {
        "controlComponents": [
          "CInputField"
        ]
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L32)
