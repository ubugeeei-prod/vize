---
title: "script/define-emits-declaration"
description: "要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式。"
---

# `script/define-emits-declaration`

要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`defineEmits(["change"])` 使用运行时数组声明；此风格规则偏好类型声明。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

## 正确示例

`defineEmits<{ change: [id: number] }>()` 将事件声明移到类型参数中，并显式描述 `emit("change", 1)` 使用的数字载荷。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-emits-declaration": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39)
