---
title: "script/prefer-computed"
description: "响应式派生状态优先使用 computed()。"
---

# `script/prefer-computed`

响应式派生状态优先使用 computed()。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

此情形要求侦听器仅派生目标值。可编辑副本及具有其他副作用的回调被允许。

## 错误示例

侦听器仅将 `count` 的派生值复制到第二个 ref `doubled`，通过手动同步维护派生状态。

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

## 正确示例

`computed(() => count.value * 2)` 直接表达派生关系，移除额外可写 ref 及其同步侦听器。

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-computed": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41)
