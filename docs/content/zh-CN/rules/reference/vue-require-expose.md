---
title: "vue/require-expose"
description: "要求 Options API 组件通过 `expose` 显式声明公开属性。"
---

# `vue/require-expose`

要求 Options API 组件通过 `expose` 显式声明公开属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

这项检查针对 Options API 的组件对象。显式声明空的 `expose` 列表也是有效的。setup 中调用 `expose` 或返回渲染函数也能满足检查；mixin 不在检查范围内。这个脚本检查不会查看并列的 `<script setup>` 块，`defineExpose` 也不是此规则的目标。

## 错误示例

这个 Options API 组件包含 `focus` 方法，却没有显式声明公开实例的接口。

```vue annotate="remove:2"
<script lang="ts">
export default { methods: { focus() {} } };
</script>
```

## 正确示例

通过 `expose` 列表显式声明，可从组件的公开实例访问 `focus`。

```vue annotate="add:2"
<script lang="ts">
export default { expose: ["focus"], methods: { focus() {} } };
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/require-expose": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_expose.rs#L24)
