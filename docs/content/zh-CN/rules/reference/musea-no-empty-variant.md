---
title: "musea/no-empty-variant"
description: "禁止空的 &lt;variant&gt; 块。"
---

# `musea/no-empty-variant`

禁止空的 &lt;variant&gt; 块。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Musea .art.vue 的 art、variant 和 style 块。

## 错误示例

名为 primary 的 variant 为空，没有提供预览内容。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

## 正确示例

variant 渲染带有 Save 内容的 primary Button。

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
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
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 与 CSS 规则](../musea-and-css.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8)
