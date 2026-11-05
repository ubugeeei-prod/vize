---
title: "script/no-get-current-instance"
description: "禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）。"
---

# `script/no-get-current-instance`

禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `opinionated` | 不支持 | 无 |

适用范围：面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本。

## 错误示例

带 Vapor 标记的 setup 导入并调用 `getCurrentInstance`，依赖了此规则针对 Vapor 组件禁止的实例 API。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

## 正确示例

`inject("app-config")` 获取显式提供的配置，无需导入或调用 `getCurrentInstance`。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
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
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38)
