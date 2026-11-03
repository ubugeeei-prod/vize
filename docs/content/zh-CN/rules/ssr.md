---
title: "SSR 规则"
description: "找出在服务端渲染和水合时会出错或行为不同的代码。"
---

# SSR 规则

找出在服务端渲染和水合时会出错或行为不同的代码。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 默认 (2)

由默认预设 `happy-path` 添加。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | 禁止 SSR 上下文中的浏览器专用全局变量。 |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | 禁止导致水合不匹配的不确定值。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
