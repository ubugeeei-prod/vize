---
title: "vue/no-deprecated-scope-attribute"
description: "禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性。"
---

# `vue/no-deprecated-scope-attribute`

禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

插槽模板通过已弃用的 scope 属性声明 props。

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

## 正确示例

默认插槽指令使用当前插槽语法声明相同的 props 绑定。

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-scope-attribute": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38)
