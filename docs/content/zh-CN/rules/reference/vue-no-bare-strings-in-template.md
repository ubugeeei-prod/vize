---
title: "vue/no-bare-strings-in-template"
description: "禁止在模板中直接写入应当国际化的用户可读文本。"
---

# `vue/no-bare-strings-in-template`

禁止在模板中直接写入应当国际化的用户可读文本。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 不支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 错误示例

可见文本和提供名称的属性直接在模板中嵌入了未翻译的字符串。

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

## 正确示例

可翻译内容调用 $t；示例中的标点和纯数字属于允许的例外。

```vue annotate="add:2,3,4,5"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<button :title="$t('close')">{{ $t('x') }}</button>
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
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L50)
