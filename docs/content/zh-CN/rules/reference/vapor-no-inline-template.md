---
title: "vapor/no-inline-template"
description: "禁止已弃用的 inline-template 属性。"
---

# `vapor/no-inline-template`

禁止已弃用的 inline-template 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

LegacyCard 为子标记使用 inline-template 属性。

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

## 正确示例

标记通过默认插槽传入，而非内联模板。

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vapor 规则](../vapor.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31)
