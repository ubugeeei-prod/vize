---
title: "script/no-options-api"
description: "禁止 Vapor 模式中的 Options API 模式。"
---

# `script/no-options-api`

禁止 Vapor 模式中的 Options API 模式。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated` | 不支持 | 无 |

适用范围：面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本。

## 错误示例

默认导出对象声明了 Options API 的 `data()`，这是此规则禁止的组件选项形式。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

## 正确示例

组件状态改为 Vapor `<script setup>` 中的 Composition API `ref`，移除 Options API 对象及其 `data` 选项。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
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
        "script/no-options-api": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43)
