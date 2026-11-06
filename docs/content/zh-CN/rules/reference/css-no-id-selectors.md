---
title: "css/no-id-selectors"
description: "不建议在 CSS 中使用 ID 选择器。"
---

# `css/no-id-selectors`

不建议在 CSS 中使用 ID 选择器。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | `opinionated`, `nuxt` | 不支持 | 无 |

适用范围：纯 CSS 的 style 块（scss、sass、less 和 stylus 块会被跳过）。

## 错误示例

`#submit` 将样式规则绑定到 ID 选择器。

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

## 正确示例

`.submit` 类提供可复用的样式入口，不使用 ID 选择器。

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "css/no-id-selectors": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 与 CSS 规则](../musea-and-css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19)
