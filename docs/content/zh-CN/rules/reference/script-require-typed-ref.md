---
title: "script/require-typed-ref"
description: "要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数。"
---

# `script/require-typed-ref`

要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

导入的 `ref` 调用既无类型参数，也无有效初始值：无参数、`null` 和 `undefined` 都无法推断未来值的预期类型。

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

## 正确示例

显式类型参数描述字符串和可空 User refs。`ref(0)` 已有具体数字初始值，可以依靠推断。

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
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
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55)
