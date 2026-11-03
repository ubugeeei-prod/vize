---
title: "musea/require-title"
description: "要求 &lt;art&gt; 块具有 title 属性。"
---

# `musea/require-title`

要求 &lt;art&gt; 块具有 title 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Musea .art.vue 的 art、variant 和 style 块。

## 错误示例

art 块标识了 Button.vue，却没有提供标题。

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

## 正确示例

defineArt 选项为 art 块提供 Button 标题。

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

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 与 CSS 规则](../musea-and-css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31)
