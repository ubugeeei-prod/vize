---
title: "script/no-computed-properties-in-data"
description: "禁止在组件的 data 函数中使用计算属性。"
---

# `script/no-computed-properties-in-data`

禁止在组件的 data 函数中使用计算属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`data()` 返回 `this.doubled`，而 `doubled` 仍是计算属性，读取发生在初始化之前。

```vue annotate="remove:4,7"
<script lang="ts">
export default {
  data() {
    return { value: this.doubled }
  },
  computed: {
    doubled() { return 1 }
  }
}
</script>
```

## 正确示例

`data()` 返回普通的 `value`，`doubled` 在初始化之后读取 `this.value`。

```vue annotate="add:4,7"
<script lang="ts">
export default {
  data() {
    return { value: 1 }
  },
  computed: {
    doubled() { return this.value * 2 }
  }
}
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
        "script/no-computed-properties-in-data": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_computed_properties_in_data.rs#L44)
