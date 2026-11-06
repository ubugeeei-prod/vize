---
title: "script/no-lifecycle-after-await"
description: "禁止在 setup() 中 await 之后注册生命周期钩子。"
---

# `script/no-lifecycle-after-await`

禁止在 setup() 中 await 之后注册生命周期钩子。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`setup()` 在 `await loadData()` 之后调用 `onMounted()`；此时没有当前组件实例，钩子不会被注册。

```vue annotate="remove:5"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    await loadData()
    onMounted(() => {})
  },
}
</script>
```

## 正确示例

`onMounted()` 在第一个 `await` 之前注册，此时 `setup()` 仍在为组件同步执行。

```vue annotate="add:6"
<script lang="ts">
import { onMounted } from 'vue'
export default {
  async setup() {
    onMounted(() => {})
    await loadData()
  },
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
      "script/no-lifecycle-after-await": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_lifecycle_after_await.rs#L45)
