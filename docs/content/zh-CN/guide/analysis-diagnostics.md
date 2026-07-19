---
title: 分析诊断
---

<!-- Generated translation; source: guide/analysis-diagnostics.md -->

# 分析诊断

本页解释了Vize诊断的组织方式。详细规则参考现存于
规则部分，让每条规则保持其行为、默认严重度、预设覆盖范围以及好坏/坏
举例一起举。

## 规则参考

- [规则概述](../rules/index.md)
- [Vue规则](../rules/vue.md)
- [无障碍规则](../rules/accessibility.md)
- [类型和文字规则](../rules/type-and-script.md)
- [HTML规则](../rules/html.md)
- [SSR规则](../rules/ssr.md)
- [蒸汽规则](../rules/vapor.md)
- [跨文件规则](../rules/cross-file.md)
- [Musea和CSS规则](../rules/musea-and-css.md)

## 诊断家族

铜绿规则是单排绒毛规则。它们使用诸如`vue/require-v-for-key`和
由`vize.config.*`、CLI、JavaScript API 和 Oxlint 桥接器配置。

跨文件诊断使用`vize:croquis/cf/*`代码。它们由
`vize lint --cross-file` Vize会建立项目图，以便比较供应商与
注入器、追踪重复的ID，并跨组件边界发现反应性危害。

类型感知诊断使用 TypeScript 检查器。他们需要的项目配置是相同的
TypeScript 能够透视`tsconfig.json`，包括 `compilerOptions.types`、`paths` 和 project。
参考资料。Vize并不要求为这些名字单独`globals`列表。

Musea和CSS诊断是库支持的规则。当 Musea 的艺术块或样式内容出现时，它们会运行
被解析并单独文档化，因为它们不属于标准的Vue模板规则
浮出水面。
