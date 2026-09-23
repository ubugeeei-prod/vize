---
title: 跨文件复杂度
---

<!-- Generated translation; source: guide/cross-file-complexity.md -->

# 跨文件复杂度

Vize 的跨文件复杂度报告是 Croquis 生成的项目图摘要。它本身不是诊断规则，而是一个可解释的分数，
下游工具可以在报告、Playground 以及未来基于阈值的检查中展示它。

该模型为 Vue 提供三种复杂度信号：

- 模板路径数：每个组件自身的圈复杂度，由 Davinci 的 S2 `template-complexity` 分析计算。它统计每个
  `v-if` / `v-else-if` 条件、每个 `v-for`，以及模板求值的表达式中的每个 `&&`、`||`、`??` 和 `?:`。
- 嵌套控制流：每个组件自身的认知复杂度。分支和循环在 `v-if`、`v-for` 和作用域插槽区域中嵌套得越深，
  代价越高。
- 组件边界数据流：props、provide/inject 和响应式边作为跨边界信号保留，而不是被压平到单个文件中。

指标定义和由语料库确定的阈值见
[`complexity-metrics.md`](https://github.com/ubugeeei-prod/vize/blob/main/docs/davinci/plan/complexity-metrics.md)。

## 分数

报告同时提供原始信号和派生分数。

| 字段              | 含义                                                                                         |
| ----------------- | -------------------------------------------------------------------------------------------- |
| `cyclomaticScore` | 所有组件自身模板圈复杂度之和。                                                               |
| `cognitiveScore`  | 所有组件自身模板认知复杂度之和。                                                             |
| `totalScore`      | 各维度分数之和：模板流、插槽、props 逐层传递、全局状态、provide/inject、透传属性和响应式图。 |
| `band`            | 面向人的分档：`low`、`moderate`、`high` 或 `extreme`。                                       |

原始输入还保留了分数背后的数字，包括：

| 信号                                                             | 为什么重要                                                           |
| ---------------------------------------------------------------- | -------------------------------------------------------------------- |
| `templateCyclomatic` 和 `templateCognitive`                      | 各组件自身模板分数的总和。                                           |
| `templateMaxNesting`                                             | 单个模板内分支、循环和作用域插槽嵌套的最大深度。                     |
| `templateScopedSlotCount`                                        | 作用域插槽耦合父子模板，因此与普通插槽分开计数。                     |
| `templateUnknown`                                                | 没有解析出 AST 的表达式（例如多语句处理函数）。它们不计入任何分数。 |
| `propDrillingEdgeCount`                                          | props 边表示跨边界的数据流。                                         |
| `provideInjectMaxDepth` 和 `provideInjectReferenceCount`         | 过深或过宽的依赖注入树会让所有权更难在本地检查。                     |
| `reactiveNodeCount`、`reactiveEdgeCount` 和 `reactiveCycleCount` | 响应式图反映声明级别的状态、副作用以及容易丢失响应性的循环。         |

## 组件边界

模板复杂度有两种视角，二者都来自同一份事实：

- **自身**（own）复杂度只看组件自己的模板。lint 规则 `vue/max-template-complexity` 按这个视角判断，
  因此把一个分支提取到子组件中，总会降低父组件的分数。
- **渲染**（rendered）复杂度是组件自身的分数，加上它渲染的每个不同组件的自身分数，沿着 Croquis
  通过 import 解析出的组件使用图计算。从两处渲染的子组件只计一次。递归组件以及相互渲染的一组组件
  也只计一次。

`CrossFileResult.templateComplexity` 列出每个组件的两种视角，渲染树最复杂的排在最前。对每个组件，
它还给出增加复杂度的结构及其行号和列号。

因此，一个看起来很浅的组件，在转发作用域插槽、逐层传递 props 或依赖很深的 provide/inject 路径时，
仍然可能得到很高的分数。Playground 的 Cross-file 模式会在诊断旁显示分数，方便在编辑 fixture 时
看到这些信号。

## lint 规则与 Doctor 发现

当组件自身模板的圈复杂度超过 11，或认知复杂度超过 16 时，`vue/max-template-complexity` 会报告一个
`warning`。这两个上限是 Vize 真实语料库中 40,724 个模板的 p95。警告指向 `<template>` 标签，并标注
增加复杂度最多的五个结构。

由于上限是 p95，大约每二十个真实组件中就有一个超出上限。因此没有任何预设启用这条规则，是否启用由
项目决定。在 `linter.rules` 中写上规则名即可启用：

```ts
export default defineConfig({
  linter: {
    rules: {
      "vue/max-template-complexity": "warn",
    },
  },
});
```

当组件的渲染复杂度超过语料库的 p95（圈复杂度 106 或认知复杂度 139）时，`vize doctor` 会以 notice
级别报告一个模板复杂度热点。

圈复杂度对每个判定加 1：每个 `v-if` / `v-else-if` 条件、每个 `v-for`，以及每个逻辑运算符和 `?:`。
认知复杂度按以下方式计算：

- `v-if` 和 `v-for` 加 1 再加上其嵌套深度。
- `v-else-if` 和 `v-else` 各加 1。
- 每一串连续的 `&&`、`||` 或 `??` 加 1。
- `?:` 加 1 再加上其嵌套深度。
- 作用域插槽的内容按多一层嵌套计算。

## 热点

报告还提供排序后的热点，这样工具可以指出产生分数的文件和组件，而不是只显示一个项目级数字。每个
热点包含本地分数输入、各维度分数、总分以及主导维度。用 `dominantDimension` 解释该条目为什么偏高，
再用 `input` 展示驱动它的原始信号。

## 当前公开接口

公开的 JSON 结构可通过 WASM 跨文件分析绑定以 `CrossFileResult.complexityReport`、
`CrossFileResult.complexityHotspots` 和 `CrossFileResult.templateComplexity` 获取。CLI 目前还不会
因为这个分数而让构建失败。请先把报告当作探索性信号，等项目有了自己的基线之后，再推广稳定的阈值。
