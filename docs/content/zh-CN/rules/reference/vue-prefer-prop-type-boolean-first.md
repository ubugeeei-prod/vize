---
title: "vue/prefer-prop-type-boolean-first"
description: "要求在组件 prop 的类型构造函数数组中将 `Boolean` 放在首位。"
---

# `vue/prefer-prop-type-boolean-first`

要求在组件 prop 的类型构造函数数组中将 `Boolean` 放在首位。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

更改构造函数顺序可能改变运行时的布尔值转换。这条按需启用的规则检查运行时的 `defineProps` 和 Options API 的 prop 类型数组；不会重排 TypeScript 联合类型。

## 错误示例

`String` 位于 `Boolean` 之前，因此不带值的 `enabled` 属性采用字符串优先的转换，而非布尔值优先的约定。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps({ enabled: [String, Boolean] });
</script>
```

## 正确示例

将 `Boolean` 放在首位，让 prop 优先进行布尔值转换，同时保留对字符串类型的支持。

```vue annotate="add:2"
<script setup lang="ts">
defineProps({ enabled: [Boolean, String] });
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
        "vue/prefer-prop-type-boolean-first": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/prefer_prop_type_boolean_first.rs#L19)
