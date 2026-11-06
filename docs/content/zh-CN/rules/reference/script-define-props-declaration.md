---
title: "script/define-props-declaration"
description: "要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式。"
---

# `script/define-props-declaration`

要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`defineProps({ title: String })` 提供运行时对象，与此规则偏好的类型 props 声明冲突。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

## 正确示例

`defineProps<{ title: string }>()` 在类型参数中声明 `title`，保留 `props.title` 访问，无需运行时声明参数。

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/define-props-declaration": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40)
