---
title: "script/no-top-level-ref-in-script"
description: "禁止顶层 ref/reactive，以防止跨请求状态污染。"
---

# `script/no-top-level-ref-in-script`

禁止顶层 ref/reactive，以防止跨请求状态污染。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

普通 `<script>` 在模块作用域初始化 `count` 和 `user`。SSR 期间，这些状态对象可能跨组件实例和请求共享。

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
  setup() {
    return { count, user }
  }
}
</script>
```

## 正确示例

setup ref 按组件实例初始化；普通脚本只保留常量、生成状态的函数，以及 `setup()` 内创建的 ref。都不会在普通模块作用域创建响应式状态。

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
  return reactive({ count: 0 })
}

export default {
  setup() {
    // Create state inside setup
    const count = ref(0)
    return { count }
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
      "script/no-top-level-ref-in-script": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63)
