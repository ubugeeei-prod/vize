---
title: "script/require-define-component"
description: "要求 TypeScript SFC 的默认导出使用 defineComponent()。"
---

# `script/require-define-component`

要求 TypeScript SFC 的默认导出使用 defineComponent()。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

TypeScript SFC 直接导出 Options API 对象，因此组件选项无法获得 `defineComponent()` 提供的上下文类型。

```vue annotate="remove:2,7"
<script lang="ts">
export default {
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
}
</script>
```

## 正确示例

用 `defineComponent()` 包裹对象后，props、emits、`this` 和其他组件选项都会获得 Vue 感知的 TypeScript 检查。

```vue annotate="add:2,3,8"
<script lang="ts">
import { defineComponent } from 'vue'
export default defineComponent({
  props: { count: Number },
  mounted() {
    console.log(this.count)
  }
})
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/require-define-component": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_define_component.rs#L48)
