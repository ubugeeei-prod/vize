---
title: "script/no-deprecated-delete-set"
description: "禁止使用 Vue 3 中已移除的 Vue 2 $set 和 $delete。"
---

# `script/no-deprecated-delete-set`

禁止使用 Vue 3 中已移除的 Vue 2 $set 和 $delete。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`created()` 调用已移除的 `this.$set` 来添加 `ready`。

```vue annotate="remove:4"
<script lang="ts">
export default {
  created() {
    this.$set(this.item, 'ready', true)
  }
}
</script>
```

## 正确示例

同一字段直接用 `this.item.ready = true` 赋值。

```vue annotate="add:4"
<script lang="ts">
export default {
  created() {
    this.item.ready = true
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
      "script/no-deprecated-delete-set": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_delete_set.rs#L38)
