---
title: "vue/max-template-depth"
description: "限制模板的嵌套深度。"
---

# `vue/max-template-depth`

限制模板的嵌套深度。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

span 的嵌套深度超过配置的上限。

```vue annotate="remove:2,3,4,5,6,7,8"
<template>
  <div>
    <section>
      <div>
        <span>deep</span>
      </div>
    </section>
  </div>
</template>
```

## 正确示例

模板只有一层元素，深度没有超过配置的上限。

```vue annotate="add:2"
<template>
  <p>ok</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/max-template-depth": "warn"
    },
    "ruleOptions": {
      "vue/max-template-depth": {
        "maxDepth": 3
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/configured/max_template_depth.rs#L13)
