---
title: "script/no-template-ref-outside-instance"
description: "禁止在没有组件实例运行的位置调用 useTemplateRef()。"
---

# `script/no-template-ref-outside-instance`

禁止在没有组件实例运行的位置调用 useTemplateRef()。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`focus()` 只作为按钮的点击处理函数运行，`setTimeout` 的回调在 setup 返回之后才运行。两者都没有当前组件实例，因此它们创建的 ref 永远不会被设置。

```vue annotate="remove:2,4,6"
<script setup lang="ts">
import { useTemplateRef } from 'vue'
function focus() {
  useTemplateRef<HTMLInputElement>('input').value?.focus()
}
setTimeout(() => useTemplateRef('input'))
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## 正确示例

`useTemplateRef('input')` 只在 setup 中调用一次。点击处理函数和 `onMounted` 回调只读取 `input.value`。

```vue annotate="add:2,3,5,7"
<script setup lang="ts">
import { onMounted, useTemplateRef } from 'vue'
const input = useTemplateRef<HTMLInputElement>('input')
function focus() {
  input.value?.focus()
}
onMounted(() => focus())
</script>
<template>
  <input ref="input" />
  <button @click="focus">Focus</button>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-template-ref-outside-instance": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_template_ref_outside_instance.rs#L78)
