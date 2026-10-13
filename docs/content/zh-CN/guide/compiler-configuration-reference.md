---
title: 编译器配置参考
---

<!-- Reviewed translation; source: guide/compiler-configuration-reference.md; scope: reference relocation -->

# 编译器配置参考

## 编译器选项

这些选项属于`compiler`。它们是基于模式支持的，并通过`defineConfig`共享;不是
每一次积分都会消耗所有领域。

| 选项                | 价值观                               | 常见用途                                |
| ------------------- | ------------------------------------ | --------------------------------------- | ------------ |
| `sourceMap`         | `boolean`                            | 在 Vite 插件                            | 中启用源映射 |
| `ssr`               | `boolean`                            | 在不依赖Vite的SSR构建标志时编译为SSR    |
| `vapor`             | `boolean`                            | 启用蒸汽模式编译                        |
| `jsxMode`           | `"vdom"`或`"vapor"`                  | `.jsx`/`.tsx`组件的默认输出后端         |
| `customRenderer`    | `boolean`                            | 将小写非HTML标签视为自定义渲染器元素    |
| `customElements`    | `string[]`                           | 作为自定义元素编译的标签模式（TresJS 用 `Tres*`） |
| `templateSyntax`    | `"standard"`、`"strict"`或`"quirks"` | 模板语法选择警告、错误或Vue-quirk处理   |
| `scriptExt`         | `"ts"`或`"js"`                       | 保留TS输出或在npm build命令中下编译为JS |
| `mode`              | `"module"`或`"function"`             | 低级别编译器输出模式                    |
| `prefixIdentifiers` | `boolean`                            | 模板标识符前缀为`_ctx`                  |
| `hoistStatic`       | `boolean`                            | 控制静态节点起重                        |
| `cacheHandlers`     | `boolean`                            | 控制事件处理缓存                        |
| `isTs`              | `boolean`                            | 解析脚本块作为TypeScript                |
| `runtimeModuleName` | `string`                             | 覆盖运行时导入模块                      |
| `runtimeGlobalName` | `string`                             | 覆盖函数/IIFE风格输出的全局运行时       |

对于 Vite 项目，直接插件选项覆盖共享配置：

```ts
import { defineConfig } from "vite";
import vize from "@vizejs/vite-plugin";

export default defineConfig({
  plugins: [
    vize({
      vapor: true,
      sourceMap: true,
      customRenderer: true,
      templateSyntax: "standard",
    }),
  ],
});
```

## 模板语法

`compiler.templateSyntax`默认是`"standard"`。

- `"standard"`接受可恢复的无效语法，发出警告，并重写为有效输出。
- `"strict"` 报告无效语法为编译错误。
- `"quirks"` 保留模板语法兼容性的怪异问题，无需额外警告。

已知的案例有：

- `v-for`带有不匹配边缘括号的别名。Vue 剥离前导`(`或后`)`
  在分`value`、`key`和`index`之前的别名;标准模式与严格模式报告
  这些别名是畸形的，而个性模式则是Vue的。
- 非空的HTML元素，采用自闭语法编写，如`<div />`或`<span />`。
  标准模式会警告并重写它们为空元素、严格模式错误和个性模式保留
  它们作为自闭叶片。

```text
<template>
  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="(item in items">{{ item }}</div>

  <!-- Standard/strict reject this. Quirk mode compiles it as `item in items`. -->
  <div v-for="item) in items">{{ item }}</div>

  <!-- Standard warns and rewrites this as `<div></div>`. Strict errors. Quirk keeps it as a leaf. -->
  <div />
</template>
```

Vue上游实现：

- [`forAliasRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/utils.ts#L571)
- [`parseForExpression`中的`stripParensRE`](https://github.com/vuejs/core/blob/main/packages/compiler-core/src/parser.ts#L493-L530)

关于 HTML 严格模式行为的 invalid 后面，请参见 [Troubleshooting](./troubleshooting.md)
自动关闭标签。

## JSX 和 TSX 输出模式

> 关于完整的创作API、作用域样式、类型检查、编辑器支持及限制，请参见
> [JSX 和 TSX 指南](./jsx.md)。本节仅涵盖输出模式配置键。

Vize 将 `.jsx`/`.tsx` Vue 组件编译为 Virtual DOM 或
[蒸汽](https://blog.vuejs.org/posts/vue-vapor)输出。`compiler.jsxMode` 选择**全局
默认**用于未明确选择加入的组件;默认是`"vdom"`。

```ts
// vize.config.ts
import { defineConfig } from "@vizejs/vite-plugin";

export default defineConfig({
  compiler: {
    // Default every .jsx/.tsx component to Vapor output.
    jsxMode: "vapor",
  },
});
```

`jsxMode`独立于`compiler.vapor`：`vapor`切换蒸汽以控制`.vue` SFC，同时`jsxMode`
控制JSX/TSX的默认后端。项目可以将SFC保留在VDOM上，同时默认JSX为
蒸汽，或者反过来。Vite 插件也直接接受 `jsxMode` 作为插件选项，这
覆盖共享配置。

### 每个组件指令

单个组件通过指令序言覆盖默认，镜像`"use strict"`：

```tsx
// Compiled to Vapor regardless of the configured default.
const Fast = () => {
  "use vue:vapor";
  return <div class="fast" />;
};

// Compiled to Virtual DOM regardless of the configured default.
const Classic = () => {
  "use vue:vdom";
  return <div class="classic" />;
};
```

由于每个组件独立路由，**单个模块可以混合两个后端**：

```tsx
// vize.config: { compiler: { jsxMode: "vapor" } }

// No directive -> takes the configured default (Vapor here).
export const Dashboard = () => <main>{/* ... */}</main>;

// Opts back into Virtual DOM just for this component.
export const LegacyWidget = () => {
  "use vue:vdom";
  return <aside>{/* ... */}</aside>;
};
```

### 优先权

组件的输出模式按以下顺序解析：

1. 每个组件的`"use vue:vapor"`/`"use vue:vdom"`指令。
2. `compiler.jsxMode`默认设置（或插件的`jsxMode`选项）。
3. 内置的备选方案，`"vdom"`。

### 诊断

一个以`"use vue:"`开头但未命名已知模式的指令（例如打字错误
`"use vue:vdomx"`）被报告为编译错误，而非无声忽略，且有两个冲突
一个组件中的模式指令（`"use vue:vapor"` 跟随 `"use vue:vdom"`）也是
被诊断出来。像`"use strict"`这样的无关序章则保持原样。

## Vue方言

`dialect` 选择 Vue 方言配置文件用于独立的 HTML 文档（`.html`/`.htm`）：

```json
{
  "dialect": "petite-vue"
}
```

- `"vue"`将独立HTML文档视为普通的Vue从CDN翻译的文档。
- `"petite-vue"` 将独立的 HTML 文档选入
  [小巧的](https://github.com/vuejs/petite-vue)方言（`v-scope`/`v-effect`
  完成和针对小范围的IDE特性）。

当缺少密钥时，会根据文档结构性地检测方言：一个`<script src>`
解析为petite-vue包、`petite-vue`的内联ES导入，或`PetiteVue.createApp`
叫。评论或散文中提到小维特从不切换方言，且单排
组件始终使用标准的Vue方言。


