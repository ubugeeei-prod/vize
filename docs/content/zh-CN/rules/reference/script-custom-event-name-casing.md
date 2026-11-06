---
title: "script/custom-event-name-casing"
description: "要求发出的自定义事件名称使用配置的格式。"
---

# `script/custom-event-name-casing`

要求发出的自定义事件名称使用配置的格式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

发出的字符串 `transitionStart` 含有大写字母，违反默认的无分隔符全小写策略。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['transitionStart'])
emit('transitionStart')         // camelCase → report
</script>
```

## 正确示例

声明和调用都使用 `transitionstart`，既保持名称一致，也遵循平台风格的全小写约定。项目仍可显式配置 camelCase 或 kebab-case。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['transitionstart'])
emit('transitionstart')
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/custom-event-name-casing": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L62)
