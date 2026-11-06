---
title: "script/prefer-import-from-vue"
description: "优先从 'vue' 导入，而不是内部包。"
---

# `script/prefer-import-from-vue`

优先从 'vue' 导入，而不是内部包。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`ref` 和 `h` 从内部 `@vue/runtime-core` 和 `@vue/runtime-dom` 包导入，而不是公共 `vue` 包。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

## 正确示例

公共辅助函数从 `vue` 导入。`vue` 未公开的 `isString` 保留 `@vue/shared`。命名空间、默认及公共与私有混合导入保留警告，但不提供自动修复。

```vue annotate="add:2,3"
<script setup lang="ts">
import { ref, h } from 'vue'
import { isString } from '@vue/shared'
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/prefer-import-from-vue": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L36)
