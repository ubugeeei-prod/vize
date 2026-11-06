---
title: "musea/require-component"
description: "要求 &lt;art&gt; 块具有 component 属性。"
---

# `musea/require-component`

要求 &lt;art&gt; 块具有 component 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Musea .art.vue 的 art、variant 和 style 块。

## 错误示例

art 块提供了标题，却未标识要预览的组件。

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

## 正确示例

defineArt 将 ./Button.vue 指定为 art 块的组件。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/require-component": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 规则](../musea.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11)
