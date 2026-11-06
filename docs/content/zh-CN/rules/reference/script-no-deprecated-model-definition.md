---
title: "script/no-deprecated-model-definition"
description: "禁止使用 Vue 3 中已移除的 Vue 2 model 选项。"
---

# `script/no-deprecated-model-definition`

禁止使用 Vue 3 中已移除的 Vue 2 model 选项。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

组件仍用 `prop` 和 `event` 声明 Vue 2 的 `model` 选项。

```vue annotate="remove:3,4,5,6"
<script lang="ts">
export default {
  model: {
    prop: 'value',
    event: 'input'
  }
}
</script>
```

## 正确示例

组件改为声明 `modelValue` prop 和 `update:modelValue` 事件。

```vue annotate="add:3,4"
<script lang="ts">
export default {
  props: ['modelValue'],
  emits: ['update:modelValue']
}
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-model-definition": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_model_definition.rs#L32)
