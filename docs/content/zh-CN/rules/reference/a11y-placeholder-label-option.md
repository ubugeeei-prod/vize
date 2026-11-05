---
title: "a11y/placeholder-label-option"
description: "要求 required 的 select 带有占位选项。"
---

# `a11y/placeholder-label-option`

要求 required 的 select 带有占位选项。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

select 设置了 `required`，但第一个选项是 Japan，表单一开始就已选中国家，`required` 永远不会阻止提交。

```vue annotate="remove:4"
<template>
  <select v-model="country" required>
    <option value="jp">Japan</option>
    <option value="fr">France</option>
  </select>
</template>
```

## 正确示例

值为空的第一个选项就是占位选项：select 从无值状态开始，`required` 会要求用户选择国家。

```vue annotate="add:3"
<template>
  <select v-model="country" required>
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L47)
