---
title: "vue/valid-v-on"
description: "要求 `v-on` 指令有效。"
---

# `vue/valid-v-on`

要求 `v-on` 指令有效。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `error` | `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated` | 不支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 模板和代码块，包括规则所需的脚本上下文。

## 修饰符验证

不受支持的修饰符会在各自名称的位置被单独报告，包括长格式 `v-on:click`、动态事件参数和监听器对象语法中的修饰符。接受的名称与 `eslint-plugin-vue` 10.9.2 一致：Vue 内置的事件修饰符、完整的按键别名表、任意单个 Unicode 码点，以及带有有限十进制整数前缀的名称（遵循 JavaScript 的 `Number.parseInt(name, 10)` 语义）。例如，此规则接受 `.13suffix`。数字键码的弃用诊断和 `.native` 的迁移诊断由其他规则负责。

只有 `.stop` 和 `.prevent` 在没有处理器时也能产生行为。`.once`、`.enter` 或配置的自定义名称等修饰符仍然需要处理器表达式。

使用自定义事件处理的项目可以在 `vue.linter.ruleOptions` 下添加名称：

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    rules: { "vue/valid-v-on": "error" },
    ruleOptions: { "vue/valid-v-on": { modifiers: ["custom"] } },
  },
});
```

这允许使用 `@click.custom="handler"`。名称必须精确匹配且区分大小写；这些选项补充内置名称，不会启用已禁用的规则。限定作用范围的配置条目可以用新列表替换继承的名称，或用 `[]` 将其重置。

按键表改编自采用 MIT 许可证的 [eslint-plugin-vue 按键别名](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json)。其源代码和许可证与实现一同保留。

## 错误示例

监听器形式缺少事件参数，或缺少所需的处理器或对象表达式。

```vue annotate="remove:2,3,4,5,6,7"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
  <div @click.sync="handleClick"></div>
  <div @click.nativeish="handleClick"></div>
  <div @click.once></div>
</template>
```

## 正确示例

提供事件及其处理器，或将监听器对象传给无参数的 `v-on`。

```vue annotate="add:2,3,4,5,6"
<template>
  <div @click="handleClick"></div>
  <form @submit.prevent></form>
  <input @keyup.page-down="handlePageDown" />
  <input @keyup.あ="handleKey" />
  <div v-on="{ click: handleClick }"></div>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  vue: {
    linter: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-on": "error"
      },
      "ruleOptions": {
        "vue/valid-v-on": {
          "modifiers": [
            "custom"
          ]
        }
      }
    },
  },
});
```

然后运行 `vp run lint`。

## 相关页面

- [Vue 规则](../vue.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30)
