---
title: "a11y/anchor-has-content"
description: "要求锚点元素具有可访问的内容。"
---

# `a11y/anchor-has-content`

要求锚点元素具有可访问的内容。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

列出其渲染子树始终提供可访问链接内容的组件名。PascalCase 与 kebab-case 写法等价。选项不会启用规则；后续配置层会替换整个列表，[] 会清除继承的名称。

## 错误示例

`/settings` 链接没有文本或其他命名内容，因此其目标没有可访问的描述。

```vue annotate="remove:2"
<template>
<a href="/app"></a>
</template>
```

## 正确示例

可见的 `Settings` 文本为同一目标链接提供内容。

```vue annotate="add:2"
<template>
<a href="/app"><RetryImage src="/badge.png" alt="Get the app" /></a>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/anchor-has-content": "warn"
    },
    "ruleOptions": {
      "a11y/anchor-has-content": {
        "accessibleChildren": [
          "RetryImage"
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
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L20)
