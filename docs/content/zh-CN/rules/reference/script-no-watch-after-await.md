---
title: "script/no-watch-after-await"
description: "禁止在 setup() 中 await 之后注册未保留停止句柄的侦听器。"
---

# `script/no-watch-after-await`

禁止在 setup() 中 await 之后注册未保留停止句柄的侦听器。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`watch()` 在 `await loadData()` 之后创建且丢弃了停止句柄，因此侦听器不会随组件一起停止。

```vue annotate="remove:5"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    await loadData()
    watch(() => props.id, reload)
  },
}
</script>
```

## 正确示例

侦听器在第一个 `await` 之前创建，因此组件卸载时 Vue 会停止它。

```vue annotate="add:6"
<script lang="ts">
import { watch } from 'vue'
export default {
  async setup(props) {
    watch(() => props.id, reload)
    await loadData()
  },
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
        "script/no-watch-after-await": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_watch_after_await.rs#L44)
