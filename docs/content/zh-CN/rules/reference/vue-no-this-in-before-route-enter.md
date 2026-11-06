---
title: "vue/no-this-in-before-route-enter"
description: "禁止在 `beforeRouteEnter` 中通过 `this` 访问尚未创建的组件实例。"
---

# `vue/no-this-in-before-route-enter`

禁止在 `beforeRouteEnter` 中通过 `this` 访问尚未创建的组件实例。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`beforeRouteEnter` 在组件实例可用之前运行，因此 `this.load()` 无法访问该实例。

```vue annotate="remove:4,5"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    this.load();
    next();
  },
  methods: { load() {} },
};
</script>
```

## 正确示例

使用传给 `next` 回调的实例，在组件创建后调用 `vm.load()`。

```vue annotate="add:4"
<script lang="ts">
export default {
  beforeRouteEnter(to, from, next) {
    next((vm) => vm.load());
  },
  methods: { load() {} },
};
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-this-in-before-route-enter": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_this_in_before_route_enter.rs#L24)
