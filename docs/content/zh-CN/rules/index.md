---
title: 规则
---

<!-- Generated translation; source: rules/index.md -->

# 规则

Vize诊断是以规则形式文档，而不是一个大型矩阵。每个规则页都保留了
检测行为接近坏/好示例，因此引用可以被当作ESLint规则来解读
手动。

## 页数

- [所有 Patina 规则](./all.md)：每个 Patina 规则实现的一页元数据表，
  包括GitHub源链接。
- [Vue规则](./vue.md)：SFC模板结构、Vue指令、组件约定，以及
  单文件 Vue 正确性检查。
- [类型和脚本规则](./type-and-script.md)：TypeScript 检查支持的诊断和 Vapor
  脚本限制。
- [HTML 规则](./html.md)：HTML 有效性和语义标记检查。
- [无障碍规则](./accessibility.md)：ARIA、键盘交互、标签、地标，以及
  无障碍媒体检查。
- [SSR规则](./ssr.md)：服务器渲染和水合危害。
- [蒸汽规则](./vapor.md)：仅蒸汽模板约束。
- [生态系统规则](./ecosystem.md)：Nuxt、Vue Router、Pinia、vue-i18n的预设支持检查，
  Vue 测试 utils，和 void vue。
- [Musea 和 CSS 规则](./musea-and-css.md)：Musea 艺术块检查和样式诊断。
- [跨文件规则](./cross-file.md)：由
  `vize lint --cross-file`。

## 预设

`essential`包含几乎应始终启用的正确性规则。`happy-path`补充道
日常Vue开发的实用卫生检查。`ecosystem`从广泛默认开始
捆绑并增加了 Vue Router、Vue I18n、Pinia、Vue 测试工具、Nuxt 和 Void Vue 检查。`nuxt`
包括以Nuxt为导向的SSR和Vapor期望。`opinionated`是最广泛的
内置预设。

`incremental`一开始是空的。当主机想选择加入特定规则但不继承
更大的预设。

## 类型感知配置

需要语义信息的规则则通过 TypeScript 项目阅读`tsconfig.json`。更喜欢
把共享环境名放在`compilerOptions.types`或项目引用里，而不是保留
在 Vize 配置中有一个独立的 `globals` 列表。
