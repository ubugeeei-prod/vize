---
title: "musea/require-title"
description: "要求 &lt;art&gt; 块具有可解析的显示标题。"
---

# `musea/require-title`

要求 &lt;art&gt; 块具有可解析的显示标题。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Musea .art.vue 的 art、variant 和 style 块。

## 错误示例

art 块既没有显式元数据，也没有可推断标题的组件。

```vue annotate="remove:2"
<art>
  <variant name="primary"><div /></variant>
</art>
```

## 正确示例

导入的 Button 目标提供推断出的显示标题。

```vue annotate="add:1,2,3,4,6"
<script setup>
import Button from "./Button.vue";
</script>

<art>
  <variant name="primary"><Button /></variant>
</art>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-title": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 规则](../musea.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L32)
