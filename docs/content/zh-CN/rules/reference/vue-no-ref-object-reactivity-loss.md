---
title: "vue/no-ref-object-reactivity-loss"
description: "禁止在同一作用域中复制 ref 的值而丢失后续响应式更新。"
---

# `vue/no-ref-object-reactivity-loss`

禁止在同一作用域中复制 ref 的值而丢失后续响应式更新。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

检查会跟踪可识别的 ref 工厂函数以及同一作用域中的值复制。其他函数内部的读取和对 `.value` 的写入不作检查。这是一条基于源码、按需启用的规则，不需要原生类型查询。

## 错误示例

在脚本根作用域将 `count.value` 读入 `snapshot`，只会保存当前数值，不能跟随后续 ref 更新。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const snapshot = count.value;
</script>
```

## 正确示例

计算属性的 getter 会在依赖变化时读取 `count.value`，保留响应式结果，避免根作用域中的值复制。

```vue annotate="add:2,4"
<script setup lang="ts">
import { computed, ref } from "vue";
const count = ref(0);
const snapshot = computed(() => count.value);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-ref-object-reactivity-loss": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_object_reactivity_loss.rs#L46)
