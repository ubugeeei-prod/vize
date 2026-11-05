---
title: "vue/valid-model-definition"
description: "要求 Vue 2 的 `model` 选项使用有效的键。"
---

# `vue/valid-model-definition`

要求 Vue 2 的 `model` 选项使用有效的键。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

这条按需启用的规则检查 Vue 2 的 `model` 选项。只接受静态键 `prop` 和 `event`，不验证它们的值。Vue 3 使用较新的组件 `v-model` 约定，而不是这个选项。

## 错误示例

`events` 不是 Vue 2 组件 `model` 选项支持的键。

```vue annotate="remove:2"
<script lang="ts">
export default { model: { prop: "value", events: "input" } };
</script>
```

## 正确示例

使用受支持的单数形式 `event` 和 `prop`，描述 Vue 2 的 model 约定。

```vue annotate="add:2"
<script lang="ts">
export default { model: { prop: "value", event: "input" } };
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
        "vue/valid-model-definition": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_model_definition.rs#L21)
