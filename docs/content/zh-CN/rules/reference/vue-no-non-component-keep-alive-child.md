---
title: "vue/no-non-component-keep-alive-child"
description: "禁止在 `<KeepAlive>` 的直属子层使用普通元素包装。"
---

# `vue/no-non-component-keep-alive-child`

禁止在 `<KeepAlive>` 的直属子层使用普通元素包装。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

KeepAlive 有条件地包装原生 div，而没有直接缓存 UserCard。

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

## 正确示例

第一个示例将 UserCard 设为条件子节点。v-show 包装层展示了不属于此条件子节点检查范围的结构，并不保证原生包装元素会被缓存。

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L26)
