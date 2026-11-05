---
title: "script/no-deprecated-events-api"
description: "禁止已移除的 Vue 2 事件 API（$on / $off / $once）。"
---

# `script/no-deprecated-events-api`

禁止已移除的 Vue 2 事件 API（$on / $off / $once）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`$on`、`$once` 和 `$off` 调用使用了 Vue 3 已移除的实例事件总线方法。

```vue annotate="remove:4,5,6,7"
<script>
export default {
  mounted() {
    this.$on('event', handler)
    const vm = this
    vm.$once('event', handler)
    this.$off('event', handler)
  }
}
</script>
```

## 正确示例

`$emit` 仍然有效，事件总线订阅则移到外部发射器的 `on` 方法。修复区分面向父组件的事件发出与外部事件总线。

```vue annotate="add:2,3,6,7"
<script>
import mitt from 'mitt'
const emitter = mitt()
export default {
  mounted() {
    this.$emit('event', payload)
    emitter.on('event', handler)
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
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L51)
