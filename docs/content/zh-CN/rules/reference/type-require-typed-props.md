---
title: "type/require-typed-props"
description: "要求 defineProps 具有类型定义。"
---

# `type/require-typed-props`

要求 defineProps 具有类型定义。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 脚本和模板中下方所示结构的类型信息。

这条规则会读取 TypeScript 项目（`tsconfig.json`）中的类型信息，因此配置中设置了 `typeAware: true`。仅设置 `typeAware` 不会启用需要单独开启的规则。

## 错误示例

仅数组形式的 `defineProps(["title"])` 只按名称声明 `title`，没有提供类型。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

## 正确示例

`defineProps<{ title: string }>()` 为 `title` 提供显式字符串类型，替代仅名称的运行时声明。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "type/require-typed-props": "warn"
    },
    "typeAware": true
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57)
