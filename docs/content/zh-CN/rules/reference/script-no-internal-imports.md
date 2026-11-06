---
title: "script/no-internal-imports"
description: "禁止从 Vue 内部模块导入。"
---

# `script/no-internal-imports`

禁止从 Vue 内部模块导入。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

两个导入都指向内部 `dist` 文件，而非 Vue 公共包入口，使组件与构建文件路径耦合。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

## 正确示例

从 `vue` 导入所需辅助函数，移除对内部发行文件位置的依赖。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/no-internal-imports": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28)
