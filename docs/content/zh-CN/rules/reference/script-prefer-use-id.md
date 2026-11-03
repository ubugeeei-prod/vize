---
title: "script/prefer-use-id"
description: "建议使用 useId() 生成唯一 ID（Vue 3.5+）。"
---

# `script/prefer-use-id`

建议使用 useId() 生成唯一 ID（Vue 3.5+）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`id` 包含 `Math.random()`，生成的输入框与标签标识符可能在服务端和客户端渲染时不同。以 ID 命名的绑定是规则可识别的生成上下文。

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## 正确示例

Vue 3.5+ 的 `useId()` 生成标识符，`:for` 和 `:id` 继续读取同一绑定，不再分别生成随机值。

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L45)
