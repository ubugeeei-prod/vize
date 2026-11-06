---
title: "vue/no-setup-props-reactivity-loss"
description: "禁止通过复制 setup 的 prop 值而丢失后续响应式更新。"
---

# `vue/no-setup-props-reactivity-loss`

禁止通过复制 setup 的 prop 值而丢失后续响应式更新。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

检查 Options API 的 `setup` 和 `<script setup>` 中根作用域的 prop 值复制。Vue 3.5 的响应式 props 解构本身仍被允许；之后在脚本根作用域复制解构出的 prop 会被报告。允许在其他函数内部读取。

## 错误示例

在脚本根作用域复制 `props.count` 会保存当前值，无法跟随后续 prop 更新。

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps({ count: Number });
const snapshot = props.count;
</script>
```

## 正确示例

在计算属性的 getter 中读取 `props.count`，让派生结果继续跟随 prop 的变化。

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps({ count: Number });
const snapshot = computed(() => props.count);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-setup-props-reactivity-loss": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_setup_props_reactivity_loss.rs#L44)
