---
title: "musea/valid-variant"
description: "要求 &lt;variant&gt; 块具有 name 属性。"
---

# `musea/valid-variant`

要求 &lt;variant&gt; 块具有 name 属性。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Musea .art.vue 的 art、variant 和 style 块。

## 错误示例

variant 省略了标识预览所需的名称。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

## 正确示例

primary 名称标识了该 variant。

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
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
        "musea/valid-variant": "error"
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 规则](../musea.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8)
