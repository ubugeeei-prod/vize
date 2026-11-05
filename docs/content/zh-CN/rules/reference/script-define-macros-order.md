---
title: "script/define-macros-order"
description: "要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序。"
---

# `script/define-macros-order`

要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`defineEmits` 位于 `defineProps` 之前，但 `<script setup>` 必须先写 `defineProps`，再写 `defineEmits`。

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineEmits before defineProps
const emit = defineEmits<{ change: [value: string] }>()
const props = defineProps<{ count: number }>()
</script>
```

## 正确示例

`defineProps` 和 `defineEmits` 依次紧跟在 import 之后、其他所有语句之前；其余宏的顺序不限。

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
import { ref } from 'vue'
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
const model = defineModel<string>()
const value = ref(0)
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
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L44)
