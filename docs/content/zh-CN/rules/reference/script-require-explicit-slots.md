---
title: "script/require-explicit-slots"
description: "要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型。"
---

# `script/require-explicit-slots`

要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

带类型的 `defineProps<{ id: number }>()` 确立了 TypeScript 语法，但 setup 使用 `useSlots()` 而没有 `defineSlots` 声明，因此规则发现缺少显式插槽约定。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

## 正确示例

`defineSlots` 声明 props 含 `msg: string` 的 `default` 插槽；`useSlots()` 现在配有显式的类型化插槽约定。

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92)
