---
title: "vue/no-restricted-call-after-await"
description: "禁止在 await 之后调用受限制的方法。"
---

# `vue/no-restricted-call-after-await`

禁止在 await 之后调用受限制的方法。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

setup 在 await 之后调用 onMounted。

```vue annotate="remove:4,5"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  async setup() {
    await Promise.resolve();
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## 正确示例

setup 在 await 之前注册 onMounted。

```vue annotate="add:4"
<script lang="ts">
import { defineComponent, onMounted } from "vue";
export default defineComponent({
  setup() {
    onMounted(() => {});
  },
});
</script>
<template>
  <p>ok</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-restricted-call-after-await": "warn"
    },
    "ruleOptions": {
      "vue/no-restricted-call-after-await": [
        {
          "module": "vue",
          "path": "onMounted"
        }
      ]
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/await_call.rs#L25)
