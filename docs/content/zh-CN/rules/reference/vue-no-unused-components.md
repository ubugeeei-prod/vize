---
title: "vue/no-unused-components"
description: "禁止注册未在模板中使用的组件。"
---

# `vue/no-unused-components`

禁止注册未在模板中使用的组件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

UserAvatar 作为组件导入，但模板从未渲染它。

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

## 正确示例

模板渲染导入的 UserAvatar，并传入 user 绑定。

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-unused-components": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L48)
