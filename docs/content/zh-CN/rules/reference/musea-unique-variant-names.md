---
title: "musea/unique-variant-names"
description: "要求 variant 名称唯一。"
---

# `musea/unique-variant-names`

要求 variant 名称唯一。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | 无 | 不支持 | 无 |

适用范围：Musea .art.vue 的 art、variant 和 style 块。

## 错误示例

同一 art 块中的两个 variant 都使用 primary 名称。

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

## 正确示例

两个 variant 分别使用不同的 primary 和 secondary 名称。

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "musea/unique-variant-names": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Musea 规则](../musea.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10)
