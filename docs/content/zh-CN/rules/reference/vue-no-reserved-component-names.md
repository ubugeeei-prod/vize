---
title: "vue/no-reserved-component-names"
description: "禁止使用保留名称作为组件名称。"
---

# `vue/no-reserved-component-names`

禁止使用保留名称作为组件名称。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

组件名称 button 与原生 HTML 元素名称冲突。

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

## 正确示例

AppButton 是应用组件名称，没有复用原生 button 名称。

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-reserved-component-names": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L43)
