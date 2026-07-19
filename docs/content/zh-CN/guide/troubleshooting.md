---
title: 故障排除
---

<!-- Generated translation; source: guide/troubleshooting.md -->

# 故障排除

## 模板语法模式

Vize默认`compiler.templateSyntax` `"standard"`。标准模式接受可恢复的模板
语法问题，报告警告，并将其重写为有效输出。

一个常见的迁移案例是非空HTML元素的自闭语法：

```vue
<template>
  <div />
  <span />
</template>
```

`<div />`和`<span />`不是有效的自闭HTML元素。标准模式将其重写为
空元素，相当于`<div></div>`和`<span></span>`，并发出警告。严格模式
报告为错误。个性模式则保持它们自动关闭，毫无预警地离开。

倾向于写明确的结尾标签：

```vue
<template>
  <div></div>
  <span></span>
</template>
```

迁移时明确选择模式：

```ts
import vize from "@vizejs/vite-plugin";

export default {
  plugins: [
    vize({
      templateSyntax: "standard",
    }),
  ],
};
```

用`"strict"`来处理语法无效时失败，或者当项目依赖Vue接受这些时`"quirks"`
标签为自动闭合叶子。有效的空元素，如`<input />`、`<img />`、`<br />`和
`<meta />`不需要个性。

## 本地类型包解析

`vize check` 在使用 bundled 之前，先解析了检查项目中的 Vue 和 Vite 类型包
备份，因此项目自身的`vue`、`@vue/runtime-dom`、`@vue`和`vite`版本驱动
生成的虚拟项目。对于不寻常的包管理器布局，设`VIZE_VUE_PACKAGE`，
`VIZE_VUE_NAMESPACE_PACKAGE`、`VIZE_VUE_RUNTIME_DOM_PACKAGE`或`VIZE_VITE_PACKAGE`露骨
包装根。`VIZE_RUNTIME_NODE_MODULES`也可以指向一个或多个`node_modules`根，作为
备用搜索路径。
