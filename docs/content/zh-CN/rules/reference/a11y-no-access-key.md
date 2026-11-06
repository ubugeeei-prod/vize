---
title: "a11y/no-access-key"
description: "禁止使用 accesskey 属性。"
---

# `a11y/no-access-key`

禁止使用 accesskey 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

`accesskey="s"` 快捷键可能与浏览器或辅助技术快捷键冲突。

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

## 正确示例

移除 `accesskey`，普通的 Save 按钮仍可使用。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/no-access-key": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19)
