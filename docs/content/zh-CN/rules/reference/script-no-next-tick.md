---
title: "script/no-next-tick"
description: "禁止面向 Vapor 的组件使用 nextTick()。"
---

# `script/no-next-tick`

禁止面向 Vapor 的组件使用 nextTick()。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本。

## 错误示例

面向 Vapor 的组件导入并等待 `nextTick`，引入此迁移规则拒绝的 DOM 刷新调度依赖。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

## 正确示例

通过 `useTemplateRef` 获取输入框，并在 `onMounted` 时聚焦。显式挂载边界替代示例中的 `nextTick` 依赖。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-next-tick": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40)
