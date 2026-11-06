---
title: "script/no-ref-as-operand"
description: "要求将 ref 绑定变量用作操作数时通过 `.value` 访问。"
---

# `script/no-ref-as-operand`

要求将 ref 绑定变量用作操作数时通过 `.value` 访问。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`count + 1` 将 ref 对象本身用作算术操作数，而不是其包裹的数字。

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

## 正确示例

`count.value + 1` 先读取包裹的数字，再加一；脚本算术需要此显式 ref 访问。

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-ref-as-operand": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41)
