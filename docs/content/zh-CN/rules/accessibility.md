---
title: "无障碍规则"
description: "检查 ARIA、键盘交互、标签、地标和媒体替代内容，让模板可以配合辅助技术使用。"
---

# 无障碍规则

检查 ARIA、键盘交互、标签、地标和媒体替代内容，让模板可以配合辅助技术使用。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 默认 (25)

由默认预设 `happy-path` 添加。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | 要求媒体元素提供替代文本。 |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | 要求锚点元素具有可访问的内容。 |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | 要求锚点元素具有有效的 href。 |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | 禁止无效的 ARIA 属性。 |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | 具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色。 |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | 禁止在不支持 ARIA 的元素上使用 ARIA 属性。 |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | 要求点击事件配有键盘事件处理器。 |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | 要求表单控件具有相关联的标签。 |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | 要求标题元素具有可访问的内容。 |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | 要求 iframe 元素具有 title 属性。 |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | 要求具有交互角色的元素可获得焦点。 |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | 要求标签具有相关联的表单控件。 |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | 要求媒体元素具有字幕。 |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | 要求鼠标事件配有 focus/blur 事件。 |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | 禁止使用 accesskey 属性。 |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | 禁止在可获得焦点的元素上使用 aria-hidden="true"。 |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | 禁止使用 autofocus 属性。 |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | 禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素。 |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | 禁止使用 &lt;i&gt; 元素表示图标。 |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | 禁止多余的 ARIA 角色。 | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | 禁止引用不存在的 ID。 |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | 禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"。 |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | 禁止在静态元素上设置事件处理器。 |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | 要求 ARIA 角色具有必需属性。 |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | 禁止正数 tabindex 值。 |  |

## 更严格的预设 (5)

只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | 禁止跳过标题层级。 | `nuxt`, `opinionated` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | 验证地标角色的位置及唯一性。 | `nuxt`, `opinionated` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | 要求 required 的 select 带有占位选项。 | `nuxt`, `opinionated` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | 建议为类似项目符号的文本使用列表元素。 | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | 要求使用 useId() 生成唯一元素 ID，而不是静态字面量。 | `nuxt`, `opinionated` |  |

## 按需启用 (1)

不属于任何预设。在传给 `withVue` 的 `linter.rules` 中按名称启用。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | 要求图片提供 alt 属性，以支持无障碍访问。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
