---
title: "script/component-options-name-casing"
description: "要求组件 `name` 选项使用 PascalCase。"
---

# `script/component-options-name-casing`

要求组件 `name` 选项使用 PascalCase。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

组件选项 `name: 'my-component'` 使用 kebab-case，但此规则要求名称字面量采用 PascalCase。

```vue annotate="remove:3"
<script lang="ts">
export default {
  name: 'my-component' // kebab-case
}
</script>
```

## 正确示例

`MyComponent` 以大写字母开头，且仅包含字母和数字，满足名称检查。

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'MyComponent'
}
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
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44)
