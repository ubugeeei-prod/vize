---
title: "script/one-component-per-file"
description: "禁止在一个文件中定义多个组件。"
---

# `script/one-component-per-file`

禁止在一个文件中定义多个组件。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 错误示例

文件导出 `Parent`，又在 `components` 里定义了内联的 `Child`。

```vue annotate="remove:3,4,5,6"
<script lang="ts">
export default {
  name: 'Parent',
  components: {
    Child: { name: 'Child' }
  }
}
</script>
```

## 正确示例

文件只定义 `Parent`。

```vue annotate="add:3"
<script lang="ts">
export default {
  name: 'Parent'
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
      "script/one-component-per-file": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/one_component_per_file.rs#L30)
