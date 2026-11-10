---
title: "vue/no-deprecated-v-on-number-modifiers"
description: "禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符。"
---

# `vue/no-deprecated-v-on-number-modifiers`

禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 支持 | 无 |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 数值修饰符的选择与修复

首个名称带十进制整数前缀的数值修饰符决定诊断结果。`@keyup.13.27="submit"` 只报告 `13`，并提供修复结果 `@keyup.enter.27="submit"`；再次修复可以将 `27` 替换为 `escape`。`@keyup.13suffix="submit"` 也会变为 `@keyup.enter="submit"`。0–9 不在检查范围内，因此 `@keyup.0.13="submit"` 不会触发此规则。`999` 等未知值会被报告，但不提供修复。现有的 Vue 2 和 petite-vue 豁免仍然有效。

## 错误示例

键盘事件处理器使用已移除的数字代码 13 和 27 来识别按键。

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

## 正确示例

处理器使用具名按键修饰符 enter 和 esc。

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "vue/no-deprecated-v-on-number-modifiers": "error"
    }
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L46)
