---
title: "Vue 规则"
description: "针对 SFC 模板、指令、组件和代码块的正确性、写法约定以及 Vue 3 迁移检查。"
---

# Vue 规则

针对 SFC 模板、指令、组件和代码块的正确性、写法约定以及 Vue 3 迁移检查。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 基础 (53)

`essential` 预设中的正确性规则。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | 支持 `<template>` 中的诊断控制注释。 |  |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | 将 JSX 元素名称中引用的变量视为已使用。 |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | 要求组件名称由多个单词组成。 |  |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | 禁止在使用 v-html 或 v-text 时提供子内容。 |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | 禁止使用已弃用的 Vue 2 管道运算符过滤器语法。 |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | 禁止在 SFC 的 `<template>` 上使用 `functional` 属性。 |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | 禁止在原生 HTML 元素上使用已弃用的 `is` 属性。 |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | 禁止使用已弃用的 `inline-template` 属性。 |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | 禁止在 &lt;router-link&gt; 上使用 `tag` prop。 |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | 禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性。 |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | 禁止使用已弃用的 `slot` 属性。 |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | 禁止使用已弃用的 `slot-scope` 属性。 |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | 禁止在 `v-bind` 上使用已弃用的 `.sync` 修饰符。 |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | 禁止使用已废弃的 `v-is` 指令。 |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | 禁止在 `v-on` 上使用已弃用的 `.native` 修饰符。 |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | 禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符。 | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | 禁止在 `v-if` / `v-else-if` 链中使用重复条件。 |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | 禁止在同一元素上重复声明属性。 |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | 禁止修改组件 props。 |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | 禁止在 `<KeepAlive>` 的直属子层使用普通元素包装。 |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | 禁止使用保留名称作为组件名称。 |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | 禁止在 `<template>` 上使用 `key` 属性。 |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | 禁止在 `<textarea>` 内使用双花括号插值。 |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | 警告可能不安全的 URL 绑定。 |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | 禁止注册未在模板中使用的组件。 |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | 禁止 v-for 和 v-slot 指令中未使用的变量定义。 |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | 禁止在同一元素上使用 `v-if` 与 `v-for`。 |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | 禁止在 `<template>` 元素上使用不起作用的属性。 |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | 禁止在 `<template v-for>` 的子节点上使用 `key`。 |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | 警告使用 v-html，以避免 XSS 漏洞。 |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | 禁止在组件元素上使用 v-text / v-html。 |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | 要求 `<component>` 元素具有 `v-bind:is`。 |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | 要求 `<transition>` 包裹的元素具有切换条件。 |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | 要求 `v-for` 指令搭配 `v-bind:key`。 |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | 存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`。 |  |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | 要求属性名称有效。 |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | 要求 `<template>` 根节点符合 Vue 3 片段语义。 |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | 要求 `v-bind` 指令有效。 |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | 要求 `v-cloak` 指令有效。 |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | 要求 `v-else` 指令有效。 | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | 要求 `v-else-if` 指令有效。 |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | 要求 `v-for` 指令有效。 |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | 要求 `v-html` 指令有效。 |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | 要求 `v-if` 指令有效。 |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | 要求 `v-is` 指令有效。 |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | 要求 `v-memo` 指令有效。 |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | 要求 `v-model` 指令有效。 |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | 要求 `v-on` 指令有效。 |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | 要求 `v-once` 指令有效。 |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | 要求 `v-pre` 指令有效。 |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | 要求 `v-show` 指令有效。 |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | 要求 `v-slot` 指令有效。 |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | 要求 `v-text` 指令有效。 |  |

## 默认 (27)

由默认预设 `happy-path` 添加。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | 规范自定义组件的属性命名风格。 | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | 要求属性保持一致的排列顺序。 |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | 要求组件定义名称使用 PascalCase 或 kebab-case。 |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | 要求显式书写 HTML 中可以省略的结束标签。 | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | 规范 HTML 属性的引号风格。 | ✓ |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | 要求双花括号插值内部的空格保持一致。 | ✓ |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | 禁止 HTML 属性使用无效的静态值。 |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | 禁止不必要的 `<template>` 元素。 |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | 禁止连续多个空格。 | ✓ |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | 禁止使用遮蔽外层作用域变量的变量名称。 |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | 禁止使用 target="_blank" 而不设置 rel="noopener noreferrer"。 |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | 禁止在 `beforeRouteEnter` 中通过 `this` 访问尚未创建的组件实例。 |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | 禁止在模板中引用未定义的变量。 |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | 要求 iframe 元素具有 sandbox 属性。 |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | 禁止 defineProps 中定义的未使用属性。 |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | 禁止从未读取的 script setup 绑定。 |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | 禁止在同一元素上使用 `v-else-if` 或 `v-else` 与 `v-for`。 |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | 要求符合 HTML 内容模型规则。 |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | 规范所声明 prop 名称的大小写风格。 |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | 要求 style 标签具有 scoped 属性。 |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | 要求 SFC 顶层元素保持一致的顺序。 |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | 建议只使用一个 style 块。 |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | 禁止在模板表达式中使用 `this.`。 |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | 规范 `v-bind` 指令风格。 | ✓ |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | 要求组件上 v-on 中的自定义事件名称使用连字符。 | ✓ |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | 规范 `v-on` 指令风格。 | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | 规范 `v-slot` 指令风格。 | ✓ |

## 更严格的预设 (27)

只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | 要求模板中的组件名称使用指定的大小写风格。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | 要求 button 元素显式指定有效的 type。 | `nuxt`, `opinionated` |  |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | 规范自闭合标签风格。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | 禁止直接将 v-for 的索引变量用作 :key。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | 禁止为 HTML 布尔属性显式指定值。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | 禁止空的 SFC 块。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | 不建议使用内联 style 属性。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | 禁止在 :class 数组绑定中使用多个对象字面量。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | 禁止在含有 v-else 的条件链中使用否定的 v-if 条件。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | 不建议使用 CSS 预处理器，优先使用现代 CSS。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | 禁止在模板的唯一根元素上使用 v-if。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | 不建议使用非标准的 script lang 值。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | 不建议在 SFC 块上使用 src 属性。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | 不建议在 template 块上使用 lang 属性。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | 报告从未在 &lt;script&gt; 中引用的模板 ref（ref="x"）。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | 禁止表达式为常量字符串字面量的双花括号插值。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | 禁止值为普通字符串字面量的 v-bind。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | 禁止 v-text 指令，优先使用双花括号插值。 | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | 建议使用 props 简写语法（Vue 3.4+）。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | 优先使用绑定到 `true` 的布尔属性简写。 | `nuxt`, `opinionated` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | 要求显式导入或注册组件。 | `opinionated` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | 建议使用 context:event 格式的作用域事件名称。 | `nuxt`, `opinionated` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | 要求渲染的插槽出口静态名称采用配置的大小写格式。 | `nuxt`, `opinionated` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | 要求使用 useId() 生成唯一元素 ID，而不是静态字面量。 | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | 要求 v-on 处理器写为方法引用。 | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | 警告 SFC 文件中的自定义块。 | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | 警告需要注册的自定义指令。 | `nuxt`, `opinionated` |  |

## 按需启用 (59)

不属于任何预设。在传给 `withVue` 的 `linter.rules` 中按名称启用。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="max-lines"></span>[`max-lines`](./reference/max-lines.md) | 限制文件的行数。 |  |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | 要求图片提供 alt 属性，以支持无障碍访问。 |  |
| <span id="vue-block-lang"></span>[`vue/block-lang`](./reference/vue-block-lang.md) | 将 SFC 块的 lang 限制为配置的语言。 |  |
| <span id="vue-camelcase"></span>[`vue/camelcase`](./reference/vue-camelcase.md) | 要求模板表达式中的标识符使用 camelCase。 |  |
| <span id="vue-dot-notation"></span>[`vue/dot-notation`](./reference/vue-dot-notation.md) | 对静态属性访问优先使用点号。 |  |
| <span id="vue-enforce-style-attribute"></span>[`vue/enforce-style-attribute`](./reference/vue-enforce-style-attribute.md) | 将 style 块限制为允许的属性模式。 |  |
| <span id="vue-eqeqeq"></span>[`vue/eqeqeq`](./reference/vue-eqeqeq.md) | 要求模板表达式使用严格相等运算符。 |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | 要求声明的组件名称与文件名匹配。 |  |
| <span id="vue-max-lines-per-block"></span>[`vue/max-lines-per-block`](./reference/vue-max-lines-per-block.md) | 限制每个 SFC 块的行数。 |  |
| <span id="vue-max-props"></span>[`vue/max-props`](./reference/vue-max-props.md) | 限制 Vue 组件的 props 数量。 |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | 限制组件自身模板的复杂度（圈复杂度和认知复杂度）。 |  |
| <span id="vue-max-template-depth"></span>[`vue/max-template-depth`](./reference/vue-max-template-depth.md) | 限制模板的嵌套深度。 |  |
| <span id="vue-next-tick-style"></span>[`vue/next-tick-style`](./reference/vue-next-tick-style.md) | 要求以 Promise 方式使用 nextTick。 |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | 禁止在模板中直接写入应当国际化的用户可读文本。 |  |
| <span id="vue-no-console"></span>[`vue/no-console`](./reference/vue-no-console.md) | 禁止模板表达式访问 console。 |  |
| <span id="vue-no-constant-condition"></span>[`vue/no-constant-condition`](./reference/vue-no-constant-condition.md) | 禁止模板表达式中的常量条件。 |  |
| <span id="vue-no-content-outside-block"></span>[`vue/no-content-outside-block`](./reference/vue-no-content-outside-block.md) | 禁止 SFC 顶层块之外的内容。 |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | 为兼容 Vue 2，禁止在组件的 `v-model` 上使用自定义修饰符。 |  |
| <span id="vue-no-define-model-default"></span>[`vue/no-define-model-default`](./reference/vue-no-define-model-default.md) | 禁止 defineModel 中的默认值。 |  |
| <span id="vue-no-empty-pattern"></span>[`vue/no-empty-pattern`](./reference/vue-no-empty-pattern.md) | 禁止模板表达式中的空解构模式。 |  |
| <span id="vue-no-implicit-coercion"></span>[`vue/no-implicit-coercion`](./reference/vue-no-implicit-coercion.md) | 禁止模板表达式中的隐式类型转换。 |  |
| <span id="vue-no-irregular-whitespace"></span>[`vue/no-irregular-whitespace`](./reference/vue-no-irregular-whitespace.md) | 检测模板表达式中的异常空白字符。 |  |
| <span id="vue-no-literals-in-template"></span>[`vue/no-literals-in-template`](./reference/vue-no-literals-in-template.md) | 禁止 v-bind 中的对象、数组和函数字面量。 |  |
| <span id="vue-no-loss-of-precision"></span>[`vue/no-loss-of-precision`](./reference/vue-no-loss-of-precision.md) | 禁止会丢失精度的数字字面量。 |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | 禁止模板包含多个根节点。 |  |
| <span id="vue-no-negated-condition"></span>[`vue/no-negated-condition`](./reference/vue-no-negated-condition.md) | 禁止以否定条件开始三元表达式。 |  |
| <span id="vue-no-props-shadow"></span>[`vue/no-props-shadow`](./reference/vue-no-props-shadow.md) | 禁止 script setup 声明遮蔽 prop。 |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | 禁止在同一作用域中复制 ref 的值而丢失后续响应式更新。 |  |
| <span id="vue-no-restricted-block"></span>[`vue/no-restricted-block`](./reference/vue-no-restricted-block.md) | 禁止使用配置的块。 |  |
| <span id="vue-no-restricted-call-after-await"></span>[`vue/no-restricted-call-after-await`](./reference/vue-no-restricted-call-after-await.md) | 禁止在 await 之后调用受限制的方法。 |  |
| <span id="vue-no-restricted-class"></span>[`vue/no-restricted-class`](./reference/vue-no-restricted-class.md) | 禁止使用配置的 class。 |  |
| <span id="vue-no-restricted-component-names"></span>[`vue/no-restricted-component-names`](./reference/vue-no-restricted-component-names.md) | 禁止使用配置的组件名。 |  |
| <span id="vue-no-restricted-component-options"></span>[`vue/no-restricted-component-options`](./reference/vue-no-restricted-component-options.md) | 禁止使用配置的组件选项。 |  |
| <span id="vue-no-restricted-custom-event"></span>[`vue/no-restricted-custom-event`](./reference/vue-no-restricted-custom-event.md) | 禁止使用配置的自定义事件。 |  |
| <span id="vue-no-restricted-html-elements"></span>[`vue/no-restricted-html-elements`](./reference/vue-no-restricted-html-elements.md) | 禁止使用配置的元素。 |  |
| <span id="vue-no-restricted-props"></span>[`vue/no-restricted-props`](./reference/vue-no-restricted-props.md) | 禁止使用配置的 props。 |  |
| <span id="vue-no-restricted-static-attribute"></span>[`vue/no-restricted-static-attribute`](./reference/vue-no-restricted-static-attribute.md) | 禁止使用配置的静态属性。 |  |
| <span id="vue-no-restricted-syntax"></span>[`vue/no-restricted-syntax`](./reference/vue-no-restricted-syntax.md) | 禁止模板表达式中配置的语法。 |  |
| <span id="vue-no-restricted-v-bind"></span>[`vue/no-restricted-v-bind`](./reference/vue-no-restricted-v-bind.md) | 禁止受限的 v-bind 参数。 |  |
| <span id="vue-no-restricted-v-on"></span>[`vue/no-restricted-v-on`](./reference/vue-no-restricted-v-on.md) | 禁止使用配置的 v-on 参数。 |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | 禁止通过复制 setup 的 prop 值而丢失后续响应式更新。 |  |
| <span id="vue-no-shadow-native-events"></span>[`vue/no-shadow-native-events`](./reference/vue-no-shadow-native-events.md) | 禁止组件事件名与原生事件冲突。 |  |
| <span id="vue-no-sparse-arrays"></span>[`vue/no-sparse-arrays`](./reference/vue-no-sparse-arrays.md) | 禁止模板表达式中的稀疏数组。 |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | 禁止静态内联 style 属性。 |  |
| <span id="vue-no-unsupported-features"></span>[`vue/no-unsupported-features`](./reference/vue-no-unsupported-features.md) | 禁止在指定的 Vue.js 版本上使用尚未支持的语法。 |  |
| <span id="vue-no-useless-concat"></span>[`vue/no-useless-concat`](./reference/vue-no-useless-concat.md) | 禁止字符串字面量之间无意义的拼接。 |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | 为兼容 Vue 2，要求将 `template v-for` 的 key 放在实际子元素上。 |  |
| <span id="vue-object-shorthand"></span>[`vue/object-shorthand`](./reference/vue-object-shorthand.md) | 要求模板对象使用属性简写。 |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | 要求在组件 prop 的类型构造函数数组中将 `Boolean` 放在首位。 |  |
| <span id="vue-prefer-template"></span>[`vue/prefer-template`](./reference/vue-prefer-template.md) | 优先使用模板字符串而非字符串拼接。 |  |
| <span id="vue-prefer-v-model"></span>[`vue/prefer-v-model`](./reference/vue-prefer-v-model.md) | 用 v-model 代替匹配的 bind 与 update 监听器。 |  |
| <span id="vue-quote-props"></span>[`vue/quote-props`](./reference/vue-quote-props.md) | 禁止不必要的带引号对象属性。 |  |
| <span id="vue-require-default-export"></span>[`vue/require-default-export`](./reference/vue-require-default-export.md) | 要求组件的 script 作为默认导出。 |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | 要求 Options API 组件通过 `expose` 显式声明公开属性。 |  |
| <span id="vue-require-macro-variable-name"></span>[`vue/require-macro-variable-name`](./reference/vue-require-macro-variable-name.md) | 要求编译器宏使用约定的变量名。 |  |
| <span id="vue-require-prop-comment"></span>[`vue/require-prop-comment`](./reference/vue-require-prop-comment.md) | 要求每个声明的 prop 都有 JSDoc 注释。 |  |
| <span id="vue-sort-keys"></span>[`vue/sort-keys`](./reference/vue-sort-keys.md) | 按配置顺序排列组件选项中的键。 |  |
| <span id="vue-v-if-else-key"></span>[`vue/v-if-else-key`](./reference/vue-v-if-else-key.md) | 要求重复的条件组件具有 key。 |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | 要求 Vue 2 的 `model` 选项使用有效的键。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
