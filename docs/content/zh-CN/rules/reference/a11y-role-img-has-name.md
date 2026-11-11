---
title: "a11y/role-img-has-name"
description: "要求 role=\"img\" 的元素具有无障碍名称。"
---

# `a11y/role-img-has-name`

要求 role="img" 的元素具有无障碍名称。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

图片在空的 `alt` 之后设置了 `role="img"`，因此会作为没有无障碍名称的图像公开。

```vue annotate="remove:2"
<template>
  <img src="/a.png" alt="" role="img" />
</template>
```

## 正确示例

`alt="Network diagram"` 为图片提供名称，因此 `role="img"` 具有无障碍名称。

```vue annotate="add:2"
<template>
  <img src="/a.png" alt="Network diagram" role="img" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "a11y/role-img-has-name": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [无障碍规则](../accessibility.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_img_has_name.rs#L14)
