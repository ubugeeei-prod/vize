---
title: "script/no-export-in-script-setup"
description: "禁止 &lt;script setup&gt; 内的 export 语句。"
---

# `script/no-export-in-script-setup`

禁止 &lt;script setup&gt; 内的 export 语句。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

`export const count` 试图从 `<script setup>` 暴露模块导出，但这里禁止运行时导出。

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

## 正确示例

移除 `export`，使 `count` 保持为 setup 绑定，而非模块导出。

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-export-in-script-setup": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49)
