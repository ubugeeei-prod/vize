---
title: "vue/no-console"
description: "禁止模板表达式访问 console。"
---

# `vue/no-console`

禁止模板表达式访问 console。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

模板事件表达式调用了 `console.log`。

```vue annotate="remove:2"
<template>
  <button @click="console.log(value)">Save</button>
</template>
```

## 正确示例

模板把日志处理交给脚本绑定。

```vue annotate="add:2"
<template>
  <button @click="logValue">Save</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-console": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/compatibility_template/core_rules/no_console.rs#L10)
