---
title: "script/no-potential-component-option-typo"
description: "标记 Options API 组件选项名称中的疑似拼写错误。"
---

# `script/no-potential-component-option-typo`

标记 Options API 组件选项名称中的疑似拼写错误。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

选项被写为 `method`，与受识别的 `methods` 选项只差一次编辑；Vue 不会将它视为预期的方法声明。

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

## 正确示例

将键改为 `methods`，使 `save()` 位于受识别的组件选项下。

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-potential-component-option-typo": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19)
