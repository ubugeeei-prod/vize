---
title: "script/consistent-props-emits-key-quotes"
description: "统一 props 和 emits 声明中带引号的键。"
---

# `script/consistent-props-emits-key-quotes`

统一 props 和 emits 声明中带引号的键。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

emits 声明将必须带引号的 `update:modelValue` 键与不带引号的 `close` 键混合使用。

```vue annotate="remove:4"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  close: []
}>()
</script>
```

## 正确示例

声明中的所有静态键都带引号，因为 `update:modelValue` 无法写成不带引号的 JavaScript 标识符。

```vue annotate="add:4,6,7,8,9"
<script setup lang="ts">
const emit = defineEmits<{
  "update:modelValue": [value: string]
  "close": []
}>()
const props = defineProps({
  "aria-label": String,
  "disabled": Boolean,
})
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-props-emits-key-quotes": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/consistent_key_quotes.rs#L49)
