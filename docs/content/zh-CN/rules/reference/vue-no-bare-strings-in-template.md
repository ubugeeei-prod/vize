---
title: "vue/no-bare-strings-in-template"
description: "禁止在模板中直接写入应当国际化的用户可读文本。"
---

# `vue/no-bare-strings-in-template`

禁止在模板中直接写入应当国际化的用户可读文本。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

指定 allowlist 时，它会替换 eslint-plugin-vue 默认的标点列表。各项是字面子字符串或使用 Rust 正则表达式语法的 `/pattern/flags` 值，并在检查剩余文本前移除；无法编译的模式会按字面量处理。省略 allowlist 可保留默认值；[] 会清空它们。选项不会启用规则。

## 错误示例

可见文本直接在模板中嵌入了未翻译的字符串。

```vue annotate="remove:2"
<template>
<p>Translate me</p>
</template>
```

## 正确示例

键盘标签来自 `$t`，而不是模板中的裸字符串。

```vue annotate="add:2"
<template>
<kbd>{{ $t("keyboard.forward") }}</kbd>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-bare-strings-in-template": "warn"
    },
    "ruleOptions": {
      "vue/no-bare-strings-in-template": {
        "allowlist": [
          "/^[DRWXYmvx]$/u"
        ]
      }
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L56)
