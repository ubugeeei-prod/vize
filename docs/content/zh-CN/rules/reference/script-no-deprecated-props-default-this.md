---
title: "script/no-deprecated-props-default-this"
description: "禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）。"
---

# `script/no-deprecated-props-default-this`

禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

prop 默认值和校验函数读取 `this`，但在 Vue 3 中这些函数不能依赖组件实例。

```vue annotate="remove:6,7,8"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // `this` is not the component instance in Vue 3.
      default() {
        return this.defaultSize
      }
    },
  }
}
</script>
```

## 正确示例

默认值从参数读取 `props.baseSize`，校验函数检查 `value` 参数。两者都不再依赖不可用的实例接收对象。

```vue annotate="add:6,7,8,11,12,13,14,15,16"
<script lang="ts">
export default {
  props: {
    size: {
      type: Number,
      // Vue 3 passes the raw props as the first argument instead.
      default(props) {
        return props.baseSize
      }
    },
    value: {
      type: Number,
      validator(value) {
        return value > 0
      }
    }
  }
}
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-deprecated-props-default-this": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L66)
