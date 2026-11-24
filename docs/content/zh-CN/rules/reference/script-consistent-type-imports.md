---
title: "script/consistent-type-imports"
description: "统一类型导入的写法，并把模板中的使用计为值使用。"
---

# `script/consistent-type-imports`

统一类型导入的写法，并把模板中的使用计为值使用。

| 默认严重程度 | 预设 | 自动修复 | 选项 |
| --- | --- | --- | --- |
| `warning` | 无 | 支持 | [规则选项](/rules/options.md) |

适用范围：Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式。

## 哪些情况计为值使用

规则通过编译器读取的同一份绑定分析，统计组件对导入的每一次使用：模板表达式、Vue 能解析的任意大小写形式的组件标签、自定义指令、模板引用、`<style>` 中的 `v-bind()`，以及另一个脚本块。

同名的 `v-for` 或插槽别名会遮蔽该导入。类型位置属于类型使用，例如模板中的 `(item as Foo)` 或 `generic` 属性。

当某个块无法读取时（外部 `src` 块、无法解析的表达式、直接调用 `eval`、Musea 的 `<variant>` 标记），规则不会对该组件报告任何问题。

模板表达式或 `<style>` 中的 `v-bind()` 对使用 `import type` 导入的名称求值时也会被报告，因为该绑定在运行时并不存在。启用 `vue/no-undefined-refs` 时，模板中的读取交由该规则报告。组件标签和指令不会被报告：已注册或自动导入的组件可以渲染它们。

## 选项

`prefer`、`fixStyle` 和 `disallowTypeAnnotations` 是 typescript-eslint 的选项。`emitDecoratorMetadata`（默认 `true`）表示项目是否输出装饰器元数据，lint 不会从 `tsconfig.json` 读取它。为 `true` 时，带装饰器的签名所序列化的类型计为值使用。

## 自动修复

自动修复只替换被报告的声明，保留下来的值说明符位置不变，因此模块的求值顺序不会改变。必须拆分且包含注释或导入属性的声明只报告，不提供修复。

## 错误示例

`Foo` 以值的形式导入，但它只用作 `defineProps` 的类型参数。`Bar` 不会被报告：模板读取了 `Bar.label`，这属于值使用。

```vue annotate="remove:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## 正确示例

`Foo` 使用 `import type` 导入，因此该导入会在构建时被擦除。`Bar` 由模板求值，所以仍然是值导入。

```vue annotate="add:3"
<script setup lang="ts">
import { Bar } from "./Foo";
import type { Foo } from "./Foo";

defineProps<{ item: Foo }>();
</script>

<template>
  <p>{{ Bar.label }} {{ item.label }}</p>
</template>
```

## 配置（Vite+）

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({}).withVue({
  linter: {
    "preset": "incremental",
    "rules": {
      "script/consistent-type-imports": "warn"
    }
  },
});
```

然后运行 `vp run lint`。

## 与 typescript-eslint 的差异

typescript-eslint 需要 `vue-eslint-parser` 才能识别模板中的使用，而 oxlint 的这条规则会跳过 `.vue` 文件。除非 `emitDecoratorMetadata` 为 `false`，否则带装饰器的签名计为值使用；typescript-eslint 会读取 `tsconfig.json`，并在这种情况下跳过所有包含装饰器的文件。自动修复不会合并到其他声明。模板表达式或 `<style>` 中的 `v-bind()` 求值的仅类型导入会被报告。

## 相关页面

- [类型与脚本规则](../type-and-script.md)
- [规则选项](/rules/options.md)
- [全部规则](../all.md)
- [源代码](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/consistent_type_imports.rs#L55)
