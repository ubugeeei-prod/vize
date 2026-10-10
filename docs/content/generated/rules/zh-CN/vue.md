---
title: Vue 规则
---

# Vue 规则

本页列出每条 Vue 规则的用途、配置、错误示例和正确示例。高亮行表示修改；复制代码时会保留完整源代码。

<span id="句法与风格规则"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [错误示例](#vue-a11y-img-alt-bad) · [正确示例](#vue-a11y-img-alt-good) | 要求图片提供 alt 属性，以支持无障碍访问 |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [错误示例](#vue-attribute-hyphenation-bad) · [正确示例](#vue-attribute-hyphenation-good) | 规范自定义组件的属性命名风格 |
| [`vue/attribute-order`](#vue-attribute-order) | [错误示例](#vue-attribute-order-bad) · [正确示例](#vue-attribute-order-good) | 要求属性保持一致的排列顺序 |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [错误示例](#vue-component-definition-name-casing-bad) · [正确示例](#vue-component-definition-name-casing-good) | 要求组件定义名称使用 PascalCase 或 kebab-case |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [错误示例](#vue-component-name-in-template-casing-bad) · [正确示例](#vue-component-name-in-template-casing-good) | 要求模板中的组件名称使用指定的大小写风格 |
| [`vue/html-button-has-type`](#vue-html-button-has-type) | [错误示例](#vue-html-button-has-type-bad) · [正确示例](#vue-html-button-has-type-good) | 要求 button 元素显式指定有效的 type |
| [`vue/html-quotes`](#vue-html-quotes) | [错误示例](#vue-html-quotes-bad) · [正确示例](#vue-html-quotes-good) | 规范 HTML 属性的引号风格 |
| [`vue/html-self-closing`](#vue-html-self-closing) | [错误示例](#vue-html-self-closing-bad) · [正确示例](#vue-html-self-closing-good) | 规范自闭合标签风格 |
| [`vue/max-template-complexity`](#vue-max-template-complexity) | [错误示例](#vue-max-template-complexity-bad) · [正确示例](#vue-max-template-complexity-good) | 限制组件自身模板的复杂度（圈复杂度和认知复杂度） |
| [`vue/multi-word-component-names`](#vue-multi-word-component-names) | [错误示例](#vue-multi-word-component-names-bad) · [正确示例](#vue-multi-word-component-names-good) | 要求组件名称由多个单词组成 |
| [`vue/mustache-interpolation-spacing`](#vue-mustache-interpolation-spacing) | [错误示例](#vue-mustache-interpolation-spacing-bad) · [正确示例](#vue-mustache-interpolation-spacing-good) | 要求双花括号插值内部的空格保持一致 |
| [`vue/no-array-index-key`](#vue-no-array-index-key) | [错误示例](#vue-no-array-index-key-bad) · [正确示例](#vue-no-array-index-key-good) | 禁止直接将 v-for 的索引变量用作 :key |
| [`vue/no-bare-strings-in-template`](#vue-no-bare-strings-in-template) | [错误示例](#vue-no-bare-strings-in-template-bad) · [正确示例](#vue-no-bare-strings-in-template-good) | 禁止在模板中直接写入应当国际化的用户可读文本 |
| [`vue/no-boolean-attr-value`](#vue-no-boolean-attr-value) | [错误示例](#vue-no-boolean-attr-value-bad) · [正确示例](#vue-no-boolean-attr-value-good) | 禁止为 HTML 布尔属性显式指定值 |
| [`vue/no-child-content`](#vue-no-child-content) | [错误示例](#vue-no-child-content-bad) · [正确示例](#vue-no-child-content-good) | 禁止在使用 v-html 或 v-text 时提供子内容 |
| [`vue/no-deprecated-filter`](#vue-no-deprecated-filter) | [错误示例](#vue-no-deprecated-filter-bad) · [正确示例](#vue-no-deprecated-filter-good) | 禁止使用已弃用的 Vue 2 管道运算符过滤器语法 |
| [`vue/no-deprecated-functional-template`](#vue-no-deprecated-functional-template) | [错误示例](#vue-no-deprecated-functional-template-bad) · [正确示例](#vue-no-deprecated-functional-template-good) | 禁止在 SFC 的 `<template>` 上使用 `functional` 属性 |
| [`vue/no-deprecated-html-element-is`](#vue-no-deprecated-html-element-is) | [错误示例](#vue-no-deprecated-html-element-is-bad) · [正确示例](#vue-no-deprecated-html-element-is-good) | 禁止在原生 HTML 元素上使用已弃用的 `is` 属性 |
| [`vue/no-deprecated-inline-template`](#vue-no-deprecated-inline-template) | [错误示例](#vue-no-deprecated-inline-template-bad) · [正确示例](#vue-no-deprecated-inline-template-good) | 禁止使用已弃用的 `inline-template` 属性 |
| [`vue/no-deprecated-router-link-tag-prop`](#vue-no-deprecated-router-link-tag-prop) | [错误示例](#vue-no-deprecated-router-link-tag-prop-bad) · [正确示例](#vue-no-deprecated-router-link-tag-prop-good) | 禁止在 &lt;router-link&gt; 上使用 `tag` prop |
| [`vue/no-deprecated-scope-attribute`](#vue-no-deprecated-scope-attribute) | [错误示例](#vue-no-deprecated-scope-attribute-bad) · [正确示例](#vue-no-deprecated-scope-attribute-good) | 禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性 |
| [`vue/no-deprecated-slot-attribute`](#vue-no-deprecated-slot-attribute) | [错误示例](#vue-no-deprecated-slot-attribute-bad) · [正确示例](#vue-no-deprecated-slot-attribute-good) | 禁止使用已弃用的 `slot` 属性 |
| [`vue/no-deprecated-slot-scope-attribute`](#vue-no-deprecated-slot-scope-attribute) | [错误示例](#vue-no-deprecated-slot-scope-attribute-bad) · [正确示例](#vue-no-deprecated-slot-scope-attribute-good) | 禁止使用已弃用的 `slot-scope` 属性 |
| [`vue/no-deprecated-v-bind-sync`](#vue-no-deprecated-v-bind-sync) | [错误示例](#vue-no-deprecated-v-bind-sync-bad) · [正确示例](#vue-no-deprecated-v-bind-sync-good) | 禁止在 `v-bind` 上使用已弃用的 `.sync` 修饰符 |
| [`vue/no-deprecated-v-on-native-modifier`](#vue-no-deprecated-v-on-native-modifier) | [错误示例](#vue-no-deprecated-v-on-native-modifier-bad) · [正确示例](#vue-no-deprecated-v-on-native-modifier-good) | 禁止在 `v-on` 上使用已弃用的 `.native` 修饰符 |
| [`vue/no-deprecated-v-on-number-modifiers`](#vue-no-deprecated-v-on-number-modifiers) | [错误示例](#vue-no-deprecated-v-on-number-modifiers-bad) · [正确示例](#vue-no-deprecated-v-on-number-modifiers-good) | 禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符 |
| [`vue/no-dupe-v-else-if`](#vue-no-dupe-v-else-if) | [错误示例](#vue-no-dupe-v-else-if-bad) · [正确示例](#vue-no-dupe-v-else-if-good) | 禁止在 `v-if` / `v-else-if` 链中使用重复条件 |
| [`vue/no-duplicate-attributes`](#vue-no-duplicate-attributes) | [错误示例](#vue-no-duplicate-attributes-bad) · [正确示例](#vue-no-duplicate-attributes-good) | 禁止在同一元素上重复声明属性 |
| [`vue/no-empty-component-block`](#vue-no-empty-component-block) | [错误示例](#vue-no-empty-component-block-bad) · [正确示例](#vue-no-empty-component-block-good) | 禁止空的 SFC 块 |
| [`vue/no-inline-style`](#vue-no-inline-style) | [错误示例](#vue-no-inline-style-bad) · [正确示例](#vue-no-inline-style-good) | 不建议使用内联 style 属性 |
| [`vue/no-invalid-html-attribute`](#vue-no-invalid-html-attribute) | [错误示例](#vue-no-invalid-html-attribute-bad) · [正确示例](#vue-no-invalid-html-attribute-good) | 禁止 HTML 属性使用无效的静态值 |
| [`vue/no-lone-template`](#vue-no-lone-template) | [错误示例](#vue-no-lone-template-bad) · [正确示例](#vue-no-lone-template-good) | 禁止不必要的 `<template>` 元素 |
| [`vue/no-multi-spaces`](#vue-no-multi-spaces) | [错误示例](#vue-no-multi-spaces-bad) · [正确示例](#vue-no-multi-spaces-good) | 禁止连续多个空格 |
| [`vue/no-multiple-objects-in-class`](#vue-no-multiple-objects-in-class) | [错误示例](#vue-no-multiple-objects-in-class-bad) · [正确示例](#vue-no-multiple-objects-in-class-good) | 禁止在 :class 数组绑定中使用多个对象字面量 |
| [`vue/no-multiple-template-root`](#vue-no-multiple-template-root) | [错误示例](#vue-no-multiple-template-root-bad) · [正确示例](#vue-no-multiple-template-root-good) | 禁止模板包含多个根节点 |
| [`vue/no-mutating-props`](#vue-no-mutating-props) | [错误示例](#vue-no-mutating-props-bad) · [正确示例](#vue-no-mutating-props-good) | 禁止修改组件 props |
| [`vue/no-negated-v-if-condition`](#vue-no-negated-v-if-condition) | [错误示例](#vue-no-negated-v-if-condition-bad) · [正确示例](#vue-no-negated-v-if-condition-good) | 禁止在含有 v-else 的条件链中使用否定的 v-if 条件 |
| [`vue/no-non-component-keep-alive-child`](#vue-no-non-component-keep-alive-child) | [错误示例](#vue-no-non-component-keep-alive-child-bad) · [正确示例](#vue-no-non-component-keep-alive-child-good) | 禁止在 `<KeepAlive>` 的直属子层使用普通元素包装 |
| [`vue/no-preprocessor-lang`](#vue-no-preprocessor-lang) | [错误示例](#vue-no-preprocessor-lang-bad) · [正确示例](#vue-no-preprocessor-lang-good) | 不建议使用 CSS 预处理器，优先使用现代 CSS |
| [`vue/no-reserved-component-names`](#vue-no-reserved-component-names) | [错误示例](#vue-no-reserved-component-names-bad) · [正确示例](#vue-no-reserved-component-names-good) | 禁止使用保留名称作为组件名称 |
| [`vue/no-root-v-if`](#vue-no-root-v-if) | [错误示例](#vue-no-root-v-if-bad) · [正确示例](#vue-no-root-v-if-good) | 禁止在模板的唯一根元素上使用 v-if |
| [`vue/no-script-non-standard-lang`](#vue-no-script-non-standard-lang) | [错误示例](#vue-no-script-non-standard-lang-bad) · [正确示例](#vue-no-script-non-standard-lang-good) | 不建议使用非标准的 script lang 值 |
| [`vue/no-src-attribute`](#vue-no-src-attribute) | [错误示例](#vue-no-src-attribute-bad) · [正确示例](#vue-no-src-attribute-good) | 不建议在 SFC 块上使用 src 属性 |
| [`vue/no-static-inline-styles`](#vue-no-static-inline-styles) | [错误示例](#vue-no-static-inline-styles-bad) · [正确示例](#vue-no-static-inline-styles-good) | 禁止静态内联 style 属性 |
| [`vue/no-template-key`](#vue-no-template-key) | [错误示例](#vue-no-template-key-bad) · [正确示例](#vue-no-template-key-good) | 禁止在 `<template>` 上使用 `key` 属性 |
| [`vue/no-template-lang`](#vue-no-template-lang) | [错误示例](#vue-no-template-lang-bad) · [正确示例](#vue-no-template-lang-good) | 不建议在 template 块上使用 lang 属性 |
| [`vue/no-template-shadow`](#vue-no-template-shadow) | [错误示例](#vue-no-template-shadow-bad) · [正确示例](#vue-no-template-shadow-good) | 禁止使用遮蔽外层作用域变量的变量名称 |
| [`vue/no-template-target-blank`](#vue-no-template-target-blank) | [错误示例](#vue-no-template-target-blank-bad) · [正确示例](#vue-no-template-target-blank-good) | 禁止使用 target="_blank" 而不设置 rel="noopener noreferrer" |
| [`vue/no-textarea-mustache`](#vue-no-textarea-mustache) | [错误示例](#vue-no-textarea-mustache-bad) · [正确示例](#vue-no-textarea-mustache-good) | 禁止在 `<textarea>` 内使用双花括号插值 |
| [`vue/no-undefined-refs`](#vue-no-undefined-refs) | [错误示例](#vue-no-undefined-refs-bad) · [正确示例](#vue-no-undefined-refs-good) | 禁止在模板中引用未定义的变量 |
| [`vue/no-unsafe-url`](#vue-no-unsafe-url) | [错误示例](#vue-no-unsafe-url-bad) · [正确示例](#vue-no-unsafe-url-good) | 警告可能不安全的 URL 绑定 |
| [`vue/no-unsandboxed-iframe`](#vue-no-unsandboxed-iframe) | [错误示例](#vue-no-unsandboxed-iframe-bad) · [正确示例](#vue-no-unsandboxed-iframe-good) | 要求 iframe 元素具有 sandbox 属性 |
| [`vue/no-unused-components`](#vue-no-unused-components) | [错误示例](#vue-no-unused-components-bad) · [正确示例](#vue-no-unused-components-good) | 禁止注册未在模板中使用的组件 |
| [`vue/no-unused-properties`](#vue-no-unused-properties) | [错误示例](#vue-no-unused-properties-bad) · [正确示例](#vue-no-unused-properties-good) | 禁止 defineProps 中定义的未使用属性 |
| [`vue/no-unused-refs`](#vue-no-unused-refs) | [错误示例](#vue-no-unused-refs-bad) · [正确示例](#vue-no-unused-refs-good) | 报告从未在 &lt;script&gt; 中引用的模板 ref（ref="x"） |
| [`vue/no-unused-setup-bindings`](#vue-no-unused-setup-bindings) | [错误示例](#vue-no-unused-setup-bindings-bad) · [正确示例](#vue-no-unused-setup-bindings-good) | 禁止从未读取的 script setup 绑定 |
| [`vue/no-unused-vars`](#vue-no-unused-vars) | [错误示例](#vue-no-unused-vars-bad) · [正确示例](#vue-no-unused-vars-good) | 禁止 v-for 和 v-slot 指令中未使用的变量定义 |
| [`vue/no-use-v-else-with-v-for`](#vue-no-use-v-else-with-v-for) | [错误示例](#vue-no-use-v-else-with-v-for-bad) · [正确示例](#vue-no-use-v-else-with-v-for-good) | 禁止在同一元素上使用 `v-else-if` 或 `v-else` 与 `v-for` |
| [`vue/no-use-v-if-with-v-for`](#vue-no-use-v-if-with-v-for) | [错误示例](#vue-no-use-v-if-with-v-for-bad) · [正确示例](#vue-no-use-v-if-with-v-for-good) | 禁止在同一元素上使用 `v-if` 与 `v-for` |
| [`vue/no-useless-mustaches`](#vue-no-useless-mustaches) | [错误示例](#vue-no-useless-mustaches-bad) · [正确示例](#vue-no-useless-mustaches-good) | 禁止表达式为常量字符串字面量的双花括号插值 |
| [`vue/no-useless-template-attributes`](#vue-no-useless-template-attributes) | [错误示例](#vue-no-useless-template-attributes-bad) · [正确示例](#vue-no-useless-template-attributes-good) | 禁止在 `<template>` 元素上使用不起作用的属性 |
| [`vue/no-useless-v-bind`](#vue-no-useless-v-bind) | [错误示例](#vue-no-useless-v-bind-bad) · [正确示例](#vue-no-useless-v-bind-good) | 禁止值为普通字符串字面量的 v-bind |
| [`vue/no-v-for-template-key-on-child`](#vue-no-v-for-template-key-on-child) | [错误示例](#vue-no-v-for-template-key-on-child-bad) · [正确示例](#vue-no-v-for-template-key-on-child-good) | 禁止在 `<template v-for>` 的子节点上使用 `key` |
| [`vue/no-v-html`](#vue-no-v-html) | [错误示例](#vue-no-v-html-bad) · [正确示例](#vue-no-v-html-good) | 警告使用 v-html，以避免 XSS 漏洞 |
| [`vue/no-v-text`](#vue-no-v-text) | [错误示例](#vue-no-v-text-bad) · [正确示例](#vue-no-v-text-good) | 禁止 v-text 指令，优先使用双花括号插值 |
| [`vue/no-v-text-v-html-on-component`](#vue-no-v-text-v-html-on-component) | [错误示例](#vue-no-v-text-v-html-on-component-bad) · [正确示例](#vue-no-v-text-v-html-on-component-good) | 禁止在组件元素上使用 v-text / v-html |
| [`vue/permitted-contents`](#vue-permitted-contents) | [错误示例](#vue-permitted-contents-bad) · [正确示例](#vue-permitted-contents-good) | 要求符合 HTML 内容模型规则 |
| [`vue/prefer-props-shorthand`](#vue-prefer-props-shorthand) | [错误示例](#vue-prefer-props-shorthand-bad) · [正确示例](#vue-prefer-props-shorthand-good) | 建议使用 props 简写语法（Vue 3.4+） |
| [`vue/prefer-true-attribute-shorthand`](#vue-prefer-true-attribute-shorthand) | [错误示例](#vue-prefer-true-attribute-shorthand-bad) · [正确示例](#vue-prefer-true-attribute-shorthand-good) | 优先使用绑定到 `true` 的布尔属性简写 |
| [`vue/prop-name-casing`](#vue-prop-name-casing) | [错误示例](#vue-prop-name-casing-bad) · [正确示例](#vue-prop-name-casing-good) | 规范所声明 prop 名称的大小写风格 |
| [`vue/require-component-is`](#vue-require-component-is) | [错误示例](#vue-require-component-is-bad) · [正确示例](#vue-require-component-is-good) | 要求 `<component>` 元素具有 `v-bind:is` |
| [`vue/require-component-registration`](#vue-require-component-registration) | [错误示例](#vue-require-component-registration-bad) · [正确示例](#vue-require-component-registration-good) | 要求显式导入或注册组件 |
| [`vue/require-scoped-style`](#vue-require-scoped-style) | [错误示例](#vue-require-scoped-style-bad) · [正确示例](#vue-require-scoped-style-good) | 要求 style 标签具有 scoped 属性 |
| [`vue/require-toggle-inside-transition`](#vue-require-toggle-inside-transition) | [错误示例](#vue-require-toggle-inside-transition-bad) · [正确示例](#vue-require-toggle-inside-transition-good) | 要求 `<transition>` 包裹的元素具有切换条件 |
| [`vue/require-v-for-key`](#vue-require-v-for-key) | [错误示例](#vue-require-v-for-key-bad) · [正确示例](#vue-require-v-for-key-good) | 要求 `v-for` 指令搭配 `v-bind:key` |
| [`vue/scoped-event-names`](#vue-scoped-event-names) | [错误示例](#vue-scoped-event-names-bad) · [正确示例](#vue-scoped-event-names-good) | 建议使用 context:event 格式的作用域事件名称 |
| [`vue/sfc-element-order`](#vue-sfc-element-order) | [错误示例](#vue-sfc-element-order-bad) · [正确示例](#vue-sfc-element-order-good) | 要求 SFC 顶层元素保持一致的顺序 |
| [`vue/single-style-block`](#vue-single-style-block) | [错误示例](#vue-single-style-block-bad) · [正确示例](#vue-single-style-block-good) | 建议只使用一个 style 块 |
| [`vue/slot-name-casing`](#vue-slot-name-casing) | [错误示例](#vue-slot-name-casing-bad) · [正确示例](#vue-slot-name-casing-good) | 要求通过 v-slot 使用的具名插槽采用 kebab-case |
| [`vue/this-in-template`](#vue-this-in-template) | [错误示例](#vue-this-in-template-bad) · [正确示例](#vue-this-in-template-good) | 禁止在模板表达式中使用 `this.` |
| [`vue/use-unique-element-ids`](#vue-use-unique-element-ids) | [错误示例](#vue-use-unique-element-ids-bad) · [正确示例](#vue-use-unique-element-ids-good) | 要求使用 useId() 生成唯一元素 ID，而不是静态字面量 |
| [`vue/use-v-on-exact`](#vue-use-v-on-exact) | [错误示例](#vue-use-v-on-exact-bad) · [正确示例](#vue-use-v-on-exact-good) | 存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact` |
| [`vue/v-bind-style`](#vue-v-bind-style) | [错误示例](#vue-v-bind-style-bad) · [正确示例](#vue-v-bind-style-good) | 规范 `v-bind` 指令风格 |
| [`vue/v-on-event-hyphenation`](#vue-v-on-event-hyphenation) | [错误示例](#vue-v-on-event-hyphenation-bad) · [正确示例](#vue-v-on-event-hyphenation-good) | 要求组件上 v-on 中的自定义事件名称使用连字符 |
| [`vue/v-on-handler-style`](#vue-v-on-handler-style) | [错误示例](#vue-v-on-handler-style-bad) · [正确示例](#vue-v-on-handler-style-good) | 要求 v-on 处理器写为方法引用或内联函数 |
| [`vue/v-on-style`](#vue-v-on-style) | [错误示例](#vue-v-on-style-bad) · [正确示例](#vue-v-on-style-good) | 规范 `v-on` 指令风格 |
| [`vue/v-slot-style`](#vue-v-slot-style) | [错误示例](#vue-v-slot-style-bad) · [正确示例](#vue-v-slot-style-good) | 规范 `v-slot` 指令风格 |
| [`vue/valid-attribute-name`](#vue-valid-attribute-name) | [错误示例](#vue-valid-attribute-name-bad) · [正确示例](#vue-valid-attribute-name-good) | 要求属性名称有效 |
| [`vue/valid-template-root`](#vue-valid-template-root) | [错误示例](#vue-valid-template-root-bad) · [正确示例](#vue-valid-template-root-good) | 要求 `<template>` 根节点符合 Vue 3 片段语义 |
| [`vue/valid-v-bind`](#vue-valid-v-bind) | [错误示例](#vue-valid-v-bind-bad) · [正确示例](#vue-valid-v-bind-good) | 要求 `v-bind` 指令有效 |
| [`vue/valid-v-cloak`](#vue-valid-v-cloak) | [错误示例](#vue-valid-v-cloak-bad) · [正确示例](#vue-valid-v-cloak-good) | 要求 `v-cloak` 指令有效 |
| [`vue/valid-v-else`](#vue-valid-v-else) | [错误示例](#vue-valid-v-else-bad) · [正确示例](#vue-valid-v-else-good) | 要求 `v-else` 指令有效 |
| [`vue/valid-v-for`](#vue-valid-v-for) | [错误示例](#vue-valid-v-for-bad) · [正确示例](#vue-valid-v-for-good) | 要求 `v-for` 指令有效 |
| [`vue/valid-v-html`](#vue-valid-v-html) | [错误示例](#vue-valid-v-html-bad) · [正确示例](#vue-valid-v-html-good) | 要求 `v-html` 指令有效 |
| [`vue/valid-v-if`](#vue-valid-v-if) | [错误示例](#vue-valid-v-if-bad) · [正确示例](#vue-valid-v-if-good) | 要求 `v-if` 指令有效 |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [错误示例](#vue-valid-v-memo-bad) · [正确示例](#vue-valid-v-memo-good) | 要求 `v-memo` 指令有效 |
| [`vue/valid-v-model`](#vue-valid-v-model) | [错误示例](#vue-valid-v-model-bad) · [正确示例](#vue-valid-v-model-good) | 要求 `v-model` 指令有效 |
| [`vue/valid-v-on`](#vue-valid-v-on) | [错误示例](#vue-valid-v-on-bad) · [正确示例](#vue-valid-v-on-good) | 要求 `v-on` 指令有效 |
| [`vue/valid-v-once`](#vue-valid-v-once) | [错误示例](#vue-valid-v-once-bad) · [正确示例](#vue-valid-v-once-good) | 要求 `v-once` 指令有效 |
| [`vue/valid-v-show`](#vue-valid-v-show) | [错误示例](#vue-valid-v-show-bad) · [正确示例](#vue-valid-v-show-good) | 要求 `v-show` 指令有效 |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [错误示例](#vue-valid-v-slot-bad) · [正确示例](#vue-valid-v-slot-good) | 要求 `v-slot` 指令有效 |
| [`vue/valid-v-text`](#vue-valid-v-text) | [错误示例](#vue-valid-v-text-bad) · [正确示例](#vue-valid-v-text-good) | 要求 `v-text` 指令有效 |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [错误示例](#vue-warn-custom-block-bad) · [正确示例](#vue-warn-custom-block-good) | 警告 SFC 文件中的自定义块 |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [错误示例](#vue-warn-custom-directive-bad) · [正确示例](#vue-warn-custom-directive-good) | 警告需要注册的自定义指令 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `vue/a11y-img-alt`

要求图片提供 alt 属性，以支持无障碍访问

[错误示例](#vue-a11y-img-alt-bad) · [正确示例](#vue-a11y-img-alt-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/a11y-img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-a11y-img-alt-bad"></span>

**错误示例**

静态图片和动态来源的图片都未提供 alt 属性。

```vue annotate="remove:2,3"
<template>
<img src="/photo.jpg" />
<img :src="photo" />
</template>
```

<span id="vue-a11y-img-alt-good"></span>

**正确示例**

承载信息的图片使用描述性 alt 文本，装饰性图片使用空 alt，动态图片则绑定其描述。

```vue annotate="add:2,3,4,5,6,7,8,9"
<template>
<!-- Informative image -->
<img src="/photo.jpg" alt="Team photo from company retreat" />

<!-- Decorative image (empty alt) -->
<img src="/decoration.svg" alt="" />

<!-- Dynamic alt -->
<img :src="photo" :alt="photoDescription" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/a11y_img_alt.rs#L33) · [全部规则](all.md)

### `vue/attribute-hyphenation`

规范自定义组件的属性命名风格

[错误示例](#vue-attribute-hyphenation-bad) · [正确示例](#vue-attribute-hyphenation-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-hyphenation-bad"></span>

**错误示例**

组件属性使用了驼峰形式 firstName。

```vue annotate="remove:2"
<template>
<UserCard firstName="Ada" />
</template>
```

<span id="vue-attribute-hyphenation-good"></span>

**正确示例**

first-name 符合配置要求的连字符组件属性命名规范。

```vue annotate="add:2"
<template>
<UserCard first-name="Ada" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_hyphenation.rs#L35) · [全部规则](all.md)

### `vue/attribute-order`

要求属性保持一致的排列顺序

[错误示例](#vue-attribute-order-bad) · [正确示例](#vue-attribute-order-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/attribute-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-attribute-order-bad"></span>

**错误示例**

事件处理器位于结构性 v-if 指令和普通 id 属性之前。

```vue annotate="remove:2"
<template>
  <div @click="onClick" v-if="show" id="main"></div>
</template>
```

<span id="vue-attribute-order-good"></span>

**正确示例**

按照规则的顺序，先写 v-if，再写 id，最后写事件处理器。

```vue annotate="add:2"
<template>
  <div v-if="show" id="main" @click="onClick"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/attribute_order.rs#L36) · [全部规则](all.md)

### `vue/component-definition-name-casing`

要求组件定义名称使用 PascalCase 或 kebab-case

[错误示例](#vue-component-definition-name-casing-bad) · [正确示例](#vue-component-definition-name-casing-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

检查的是组件文件名。接受 PascalCase 和 kebab-case；混合大小写会被报告。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-definition-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-definition-name-casing-bad"></span>

**错误示例**

文件名 myComponent.vue 以小写字母开头，却在内部使用大写字母，既不是 PascalCase，也不是 kebab-case。

`myComponent.vue`

```vue
<template><p>Content</p></template>
```

<span id="vue-component-definition-name-casing-good"></span>

**正确示例**

将文件重命名为 MyComponent.vue 即采用 PascalCase；模板内容无需更改。

`MyComponent.vue`

```vue
<template><p>Content</p></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/component_definition_name_casing.rs#L36) · [全部规则](all.md)

### `vue/component-name-in-template-casing`

要求模板中的组件名称使用指定的大小写风格

[错误示例](#vue-component-name-in-template-casing-bad) · [正确示例](#vue-component-name-in-template-casing-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/component-name-in-template-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-component-name-in-template-casing-bad"></span>

**错误示例**

配置要求 PascalCase，但组件名称使用了 kebab-case 和 camelCase。

```vue annotate="remove:5,6"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <my-component />
  <myComponent />
</template>
```

<span id="vue-component-name-in-template-casing-good"></span>

**正确示例**

MyComponent 使用 PascalCase；原生 slot 语法仍使用小写。

```vue annotate="add:5,6,7"
<script setup>
import MyComponent from "./MyComponent.vue";
</script>
<template>
  <MyComponent />
  <RouterView />
  <slot />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/component_name_in_template_casing.rs#L31) · [全部规则](all.md)

### `vue/html-button-has-type`

要求 button 元素显式指定有效的 type

[错误示例](#vue-html-button-has-type-bad) · [正确示例](#vue-html-button-has-type-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-button-has-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-button-has-type-bad"></span>

**错误示例**

一个按钮省略了 type，另一个使用了不受支持的 foo 类型。

```vue annotate="remove:2,3"
<template>
<button>Click</button>
<button type="foo">Click</button>
</template>
```

<span id="vue-html-button-has-type-good"></span>

**正确示例**

按钮指定 button、submit 或 reset；绑定的 type 被视为动态值。

```vue annotate="add:2,3,4,5"
<template>
<button type="button">Click</button>
<button type="submit">Save</button>
<button type="reset">Reset</button>
<button :type="dynamicType">Click</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_button_has_type.rs#L39) · [全部规则](all.md)

### `vue/html-quotes`

规范 HTML 属性的引号风格

[错误示例](#vue-html-quotes-bad) · [正确示例](#vue-html-quotes-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-quotes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-quotes-bad"></span>

**错误示例**

属性使用单引号或不加引号，不符合双引号规范。

```vue annotate="remove:2,3,4"
<template>
  <div class='foo'></div>
  <div class=foo></div>
  <div v-if='ready'></div>
</template>
```

<span id="vue-html-quotes-good"></span>

**正确示例**

普通属性和指令表达式都使用双引号。

```vue annotate="add:2,3"
<template>
  <div class="foo"></div>
  <div v-if="ready"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/html_quotes.rs#L53) · [全部规则](all.md)

### `vue/html-self-closing`

规范自闭合标签风格

[错误示例](#vue-html-self-closing-bad) · [正确示例](#vue-html-self-closing-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/html-self-closing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-html-self-closing-bad"></span>

**错误示例**

空组件使用成对的起止标签，而空元素 img 和 br 没有采用配置要求的自闭合写法。

```vue annotate="remove:2,3,4"
<template>
  <MyComponent></MyComponent>
  <img>
  <br>
</template>
```

<span id="vue-html-self-closing-good"></span>

**正确示例**

组件和空元素使用自闭合语法；包含内容的 div 保留结束标签。

```vue annotate="add:2,3,4,5,6,7"
<template>
  <MyComponent />
  <div></div>
  <div />
  <img />
  <br />
  <div>content</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/html_self_closing.rs#L30) · [全部规则](all.md)

### `vue/max-template-complexity`

限制组件自身模板的复杂度（圈复杂度和认知复杂度）

[错误示例](#vue-max-template-complexity-bad) · [正确示例](#vue-max-template-complexity-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

Bad 的圈复杂度为 13，认知复杂度为 25（上限分别为 11 和 16）。每个组件单独计量；仅支持内联 HTML 模板。

参见[复杂度评分与组件边界](../guide/cross-file-complexity.md)，了解示例中两项评分的各部分贡献。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/max-template-complexity": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-max-template-complexity-bad"></span>

**错误示例**

父组件编写的分支、循环、插槽内容和表达式中的判断产生了 13 和 25 的复杂度，超过默认上限 11 和 16。

```vue annotate="remove:1,2,3,5,6,7,8,9,10,11,12,13,14,15,16,17,18,19"
<script setup lang="ts">
defineProps<{ rows: Row[] }>();
</script>
<template>
  <section>
    <h1>{{ user ? user.name : 'Guest' }}</h1>
    <DataTable :rows="rows">
      <template #cell="{ row, column }">
        <span v-if="column.key === 'status'" :class="row.active ? 'on' : 'off'">{{ row.status ?? 'unknown' }}</span>
        <a v-else-if="column.key === 'link' && row.url" :href="row.url">{{ row.label }}</a>
        <template v-else>
          <em v-for="tag in row.tags" :key="tag">
            <b v-if="tag.pinned || tag.starred">{{ tag.hot ? '!' : '' }}</b>
          </em>
        </template>
      </template>
    </DataTable>
    <p v-if="!rows.length && !loading">No data</p>
  </section>
</template>
```

<span id="vue-max-template-complexity-good"></span>

**正确示例**

父组件模板将渲染交给 RowList，只保留一个 v-if；其自身的复杂度为 2 和 1。

```vue annotate="add:2"
<template>
  <RowList v-if="ready" :rows="rows" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/max_template_complexity.rs#L56) · [全部规则](all.md)

### `vue/multi-word-component-names`

要求组件名称由多个单词组成

[错误示例](#vue-multi-word-component-names-bad) · [正确示例](#vue-multi-word-component-names-good)

默认严重程度: `error`  
预设: `essential`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

问题在于文件名。需要重命名该组件；更改子标签不能修复此问题。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/multi-word-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-multi-word-component-names-bad"></span>

**错误示例**

Item.vue 使用了单个单词的组件名称。

`Item.vue`

```vue
<template><p>Item</p></template>
```

<span id="vue-multi-word-component-names-good"></span>

**正确示例**

TodoItem.vue 为同一模板使用了多个单词组成的组件名称。

`TodoItem.vue`

```vue
<template><p>Item</p></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/multi_word_component_names.rs#L34) · [全部规则](all.md)

### `vue/mustache-interpolation-spacing`

要求双花括号插值内部的空格保持一致

[错误示例](#vue-mustache-interpolation-spacing-bad) · [正确示例](#vue-mustache-interpolation-spacing-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/mustache-interpolation-spacing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-mustache-interpolation-spacing-bad"></span>

**错误示例**

文本插值在一个或两个分隔符边界处缺少空格。

```vue annotate="remove:2,3,4"
<template>
  <div>{{text}}</div>
  <div>{{ text}}</div>
  <div>{{text }}</div>
</template>
```

<span id="vue-mustache-interpolation-spacing-good"></span>

**正确示例**

表达式与双花括号的起止分隔符之间均有空格。

```vue annotate="add:2,3,4"
<template>
  <div>{{ text }}</div>
  <div>{{ foo.bar }}</div>
  <div>{{ foo + bar }}</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/mustache_interpolation_spacing.rs#L35) · [全部规则](all.md)

### `vue/no-array-index-key`

禁止直接将 v-for 的索引变量用作 :key

[错误示例](#vue-no-array-index-key-bad) · [正确示例](#vue-no-array-index-key-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-array-index-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-array-index-key-bad"></span>

**错误示例**

列表使用当前索引作为 key，因此重新排列列表时，列表项的身份会发生变化。

```vue annotate="remove:2"
<template>
<li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
</template>
```

<span id="vue-no-array-index-key-good"></span>

**正确示例**

key 来自 item.id，即使位置改变，也能保持每个列表项的身份。

```vue annotate="add:2"
<template>
<li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_array_index_key.rs#L32) · [全部规则](all.md)

### `vue/no-bare-strings-in-template`

禁止在模板中直接写入应当国际化的用户可读文本

[错误示例](#vue-no-bare-strings-in-template-bad) · [正确示例](#vue-no-bare-strings-in-template-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-bare-strings-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-bare-strings-in-template-bad"></span>

**错误示例**

可见文本和提供名称的属性直接在模板中嵌入了未翻译的字符串。

```vue annotate="remove:2,3,4,5"
<template>
<div>hello</div>
<img alt="a cat" />
<input placeholder="Search" />
<button title="Close">x</button>
</template>
```

<span id="vue-no-bare-strings-in-template-good"></span>

**正确示例**

可翻译内容调用 $t；示例中的标点和纯数字属于允许的例外。

```vue annotate="add:2,3,4,5,6"
<template>
<div>{{ $t('hello') }}</div>
<img :alt="$t('cat')" />
<div>-</div>
<div>123</div>
<button :title="$t('close')">{{ $t('x') }}</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_bare_strings_in_template.rs#L47) · [全部规则](all.md)

### `vue/no-boolean-attr-value`

禁止为 HTML 布尔属性显式指定值

[错误示例](#vue-no-boolean-attr-value-bad) · [正确示例](#vue-no-boolean-attr-value-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-boolean-attr-value": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-boolean-attr-value-bad"></span>

**错误示例**

布尔属性 disabled 和 checked 包含了多余的字符串值。

```vue annotate="remove:2,3,4"
<template>
  <input disabled="disabled" />
  <input checked="checked" />
  <button disabled="true">Save</button>
</template>
```

<span id="vue-no-boolean-attr-value-good"></span>

**正确示例**

只要存在这些布尔属性，就能表达相同的启用状态，无需指定值。

```vue annotate="add:2,3,4"
<template>
  <input disabled />
  <input checked />
  <button disabled>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_boolean_attr_value.rs#L36) · [全部规则](all.md)

### `vue/no-child-content`

禁止在使用 v-html 或 v-text 时提供子内容

[错误示例](#vue-no-child-content-bad) · [正确示例](#vue-no-child-content-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-child-content": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-child-content-bad"></span>

**错误示例**

v-text 会替换段落内容，因此编写的后备文本无法在该指令生效后保留。

```vue annotate="remove:2"
<template>
  <p v-text="message">Fallback text</p>
</template>
```

<span id="vue-no-child-content-good"></span>

**正确示例**

移除子文本后，v-text 成为段落内容的唯一来源。

```vue annotate="add:2"
<template>
  <p v-text="message" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_child_content.rs#L30) · [全部规则](all.md)

### `vue/no-deprecated-filter`

禁止使用已弃用的 Vue 2 管道运算符过滤器语法

[错误示例](#vue-no-deprecated-filter-bad) · [正确示例](#vue-no-deprecated-filter-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-filter": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-filter-bad"></span>

**错误示例**

管道使用了已移除的 Vue 过滤器语法来应用 capitalize。

```vue annotate="remove:2"
<template>
{{ message | capitalize }}
</template>
```

<span id="vue-no-deprecated-filter-good"></span>

**正确示例**

调用 capitalize(message)，以普通表达式完成转换。

```vue annotate="add:2"
<template>
{{ capitalize(message) }}
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_filter.rs#L53) · [全部规则](all.md)

### `vue/no-deprecated-functional-template`

禁止在 SFC 的 `<template>` 上使用 `functional` 属性

[错误示例](#vue-no-deprecated-functional-template-bad) · [正确示例](#vue-no-deprecated-functional-template-good)

默认严重程度: `error`  
预设: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-functional-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-functional-template-bad"></span>

**错误示例**

SFC 模板使用了已移除的 functional 属性，并读取旧的 props 上下文。

```vue annotate="remove:1,2"
<template functional>
  <div>{{ props.msg }}</div>
</template>
```

<span id="vue-no-deprecated-functional-template-good"></span>

**正确示例**

普通模板省略 functional，直接读取组件绑定 msg。

```vue annotate="add:1,2"
<template>
  <div>{{ msg }}</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_functional_template.rs#L57) · [全部规则](all.md)

### `vue/no-deprecated-html-element-is`

禁止在原生 HTML 元素上使用已弃用的 `is` 属性

[错误示例](#vue-no-deprecated-html-element-is-bad) · [正确示例](#vue-no-deprecated-html-element-is-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-html-element-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-html-element-is-bad"></span>

**错误示例**

原生 div 使用了没有前缀的旧 is 属性来指定 Vue 组件。

```vue annotate="remove:2"
<template>
  <div is="MyComponent" />
</template>
```

<span id="vue-no-deprecated-html-element-is-good"></span>

**正确示例**

动态组件使用 :is；原生元素的写法则显式使用 vue: 前缀。

```vue annotate="add:2,3"
<template>
  <component :is="MyComponent" />
  <div is="vue:MyComponent" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_html_element_is.rs#L39) · [全部规则](all.md)

### `vue/no-deprecated-inline-template`

禁止使用已弃用的 `inline-template` 属性

[错误示例](#vue-no-deprecated-inline-template-bad) · [正确示例](#vue-no-deprecated-inline-template-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-inline-template-bad"></span>

**错误示例**

Card 使用已弃用的 inline-template 属性接收其内容。

```vue annotate="remove:2"
<template>
<Card inline-template><p>Details</p></Card>
</template>
```

<span id="vue-no-deprecated-inline-template-good"></span>

**正确示例**

省略 inline-template 属性，以通常方式传入相同内容。

```vue annotate="add:2"
<template>
<Card><p>Details</p></Card>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_inline_template.rs#L20) · [全部规则](all.md)

### `vue/no-deprecated-router-link-tag-prop`

禁止在 &lt;router-link&gt; 上使用 `tag` prop

[错误示例](#vue-no-deprecated-router-link-tag-prop-bad) · [正确示例](#vue-no-deprecated-router-link-tag-prop-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-router-link-tag-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

**错误示例**

RouterLink 使用了已移除的 tag prop 来指定 button 元素。

```vue annotate="remove:2"
<template>
  <router-link to="/home" tag="button">Home</router-link>
</template>
```

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

**正确示例**

插槽将 navigate 提供给显式编写的 button。

```vue annotate="add:2,3,4"
<template>
  <router-link to="/home" v-slot="{ navigate }">
    <button @click="navigate">Home</button>
  </router-link>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_router_link_tag_prop.rs#L37) · [全部规则](all.md)

### `vue/no-deprecated-scope-attribute`

禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性

[错误示例](#vue-no-deprecated-scope-attribute-bad) · [正确示例](#vue-no-deprecated-scope-attribute-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-scope-attribute-bad"></span>

**错误示例**

插槽模板通过已弃用的 scope 属性声明 props。

```vue annotate="remove:2"
<template>
<Card><template scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-scope-attribute-good"></span>

**正确示例**

默认插槽指令使用当前插槽语法声明相同的 props 绑定。

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_scope_attribute.rs#L38) · [全部规则](all.md)

### `vue/no-deprecated-slot-attribute`

禁止使用已弃用的 `slot` 属性

[错误示例](#vue-no-deprecated-slot-attribute-bad) · [正确示例](#vue-no-deprecated-slot-attribute-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-attribute-bad"></span>

**错误示例**

通过旧 slot 属性选择 header 插槽。

```vue annotate="remove:3,4"
<template>
  <Foo>
    <template slot="header"><h1>Title</h1></template>
    <div :slot="name">Title</div>
  </Foo>
</template>
```

<span id="vue-no-deprecated-slot-attribute-good"></span>

**正确示例**

v-slot:header 使用当前指令显式选择 header 插槽。

```vue annotate="add:3"
<template>
  <Foo>
    <template v-slot:header><h1>Title</h1></template>
  </Foo>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_attribute.rs#L39) · [全部规则](all.md)

### `vue/no-deprecated-slot-scope-attribute`

禁止使用已弃用的 `slot-scope` 属性

[错误示例](#vue-no-deprecated-slot-scope-attribute-bad) · [正确示例](#vue-no-deprecated-slot-scope-attribute-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-slot-scope-attribute": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

**错误示例**

模板通过已弃用的 slot-scope 属性接收插槽 props。

```vue annotate="remove:2"
<template>
<Card><template slot-scope="props">{{ props.name }}</template></Card>
</template>
```

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

**正确示例**

#default 指令接收这些 props，无需使用 slot-scope。

```vue annotate="add:2"
<template>
<Card><template #default="props">{{ props.name }}</template></Card>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_slot_scope_attribute.rs#L33) · [全部规则](all.md)

### `vue/no-deprecated-v-bind-sync`

禁止在 `v-bind` 上使用已弃用的 `.sync` 修饰符

[错误示例](#vue-no-deprecated-v-bind-sync-bad) · [正确示例](#vue-no-deprecated-v-bind-sync-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-bind-sync": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

**错误示例**

绑定使用了已移除的 .sync 修饰符，包括它与 .camel 的组合。

```vue annotate="remove:2,3,4"
<template>
<MyComponent :title.sync="title" />
<MyComponent v-bind:title.sync="title" />
<MyComponent :title.sync.camel="title" />
</template>
```

<span id="vue-no-deprecated-v-bind-sync-good"></span>

**正确示例**

使用普通的单向 title 绑定；需要更新通道时，则使用 v-model:title。

```vue annotate="add:2,3"
<template>
<MyComponent :title="title" />
<MyComponent v-model:title="title" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_bind_sync.rs#L42) · [全部规则](all.md)

### `vue/no-deprecated-v-on-native-modifier`

禁止在 `v-on` 上使用已弃用的 `.native` 修饰符

[错误示例](#vue-no-deprecated-v-on-native-modifier-bad) · [正确示例](#vue-no-deprecated-v-on-native-modifier-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-native-modifier": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

**错误示例**

组件事件处理器使用了已移除的 .native 事件修饰符。

```vue annotate="remove:2,3,4"
<template>
<MyComponent @click.native="handler" />
<MyComponent v-on:click.native="handler" />
<MyComponent @click.native.stop="handler" />
</template>
```

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

**正确示例**

处理器省略 .native，并保留 .stop 等其他事件修饰符。

```vue annotate="add:2,3"
<template>
<MyComponent @click="handler" />
<MyComponent @click.stop="handler" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_native_modifier.rs#L43) · [全部规则](all.md)

### `vue/no-deprecated-v-on-number-modifiers`

禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符

[错误示例](#vue-no-deprecated-v-on-number-modifiers-bad) · [正确示例](#vue-no-deprecated-v-on-number-modifiers-good)

默认严重程度: `error`  
预设: `ecosystem`, `essential`, `happy-path`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-deprecated-v-on-number-modifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

**错误示例**

键盘事件处理器使用已移除的数字代码 13 和 27 来识别按键。

```vue annotate="remove:2,3,4"
<template>
<input @keyup.13="submit" />
<input v-on:keyup.27="cancel" />
<input @keyup.13.stop="submit" />
</template>
```

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

**正确示例**

处理器使用具名按键修饰符 enter 和 esc。

```vue annotate="add:2,3"
<template>
<input @keyup.enter="submit" />
<input @keyup.esc="cancel" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_deprecated_v_on_number_modifiers.rs#L43) · [全部规则](all.md)

### `vue/no-dupe-v-else-if`

禁止在 `v-if` / `v-else-if` 链中使用重复条件

[错误示例](#vue-no-dupe-v-else-if-bad) · [正确示例](#vue-no-dupe-v-else-if-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-dupe-v-else-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-dupe-v-else-if-bad"></span>

**错误示例**

else-if 重复了首个分支已检查的 ready 条件，导致后面的分支无法执行。

```vue annotate="remove:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'ready'">Still ready</p>
</template>
```

<span id="vue-no-dupe-v-else-if-good"></span>

**正确示例**

第二个分支检查 loading，这是不同的状态，因此能够进入 else-if。

```vue annotate="add:3"
<template>
  <p v-if="status === 'ready'">Ready</p>
  <p v-else-if="status === 'loading'">Loading</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_dupe_v_else_if.rs#L34) · [全部规则](all.md)

### `vue/no-duplicate-attributes`

禁止在同一元素上重复声明属性

[错误示例](#vue-no-duplicate-attributes-bad) · [正确示例](#vue-no-duplicate-attributes-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-duplicate-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-duplicate-attributes-bad"></span>

**错误示例**

同一个按钮声明了两次 class，而不是将它们合并为一个 class 值。

```vue annotate="remove:2"
<template>
  <button class="primary" class="large">Save</button>
</template>
```

<span id="vue-no-duplicate-attributes-good"></span>

**正确示例**

两个类名都放在同一个 class 属性中。

```vue annotate="add:2"
<template>
  <button class="primary large">Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_duplicate_attributes.rs#L31) · [全部规则](all.md)

### `vue/no-empty-component-block`

禁止空的 SFC 块

[错误示例](#vue-no-empty-component-block-bad) · [正确示例](#vue-no-empty-component-block-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-empty-component-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-empty-component-block-bad"></span>

**错误示例**

template、script 和 style 块都没有实质内容。

```vue annotate="remove:1,3,5"
<template></template>

<script></script>

<style>
</style>
```

<span id="vue-no-empty-component-block-good"></span>

**正确示例**

每个保留的块都包含实际的标记、脚本声明或样式声明。

```vue annotate="add:1,2,3,5,6,7,9,10"
<template>
  <div>Hello</div>
</template>

<script setup>
const message = "Hello";
</script>

<style scoped>
.button { color: red; }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_empty_component_block.rs#L42) · [全部规则](all.md)

### `vue/no-inline-style`

不建议使用内联 style 属性

[错误示例](#vue-no-inline-style-bad) · [正确示例](#vue-no-inline-style-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-inline-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-inline-style-bad"></span>

**错误示例**

静态 style 属性将颜色声明直接嵌入元素。

```vue annotate="remove:2"
<template>
  <div style="color: red">Text</div>
</template>
```

<span id="vue-no-inline-style-good"></span>

**正确示例**

类名表达固定颜色；依赖 ratio 的宽度仍使用动态样式绑定，不属于静态属性检查的范围。

```vue annotate="add:2,3,4"
<template>
  <div class="text-red">Text</div>
  <span :class="{ 'text-red': isRed }">Text</span>
  <div :style="{ width: `${ratio}%` }">Text</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_inline_style.rs#L33) · [全部规则](all.md)

### `vue/no-invalid-html-attribute`

禁止 HTML 属性使用无效的静态值

[错误示例](#vue-no-invalid-html-attribute-bad) · [正确示例](#vue-no-invalid-html-attribute-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-invalid-html-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-invalid-html-attribute-bad"></span>

**错误示例**

锚点元素将 stylesheet 用作 rel 值，但该值应当用于样式表 link 元素。

```vue annotate="remove:2"
<template>
<a href="/guide" rel="stylesheet">Guide</a>
</template>
```

<span id="vue-no-invalid-html-attribute-good"></span>

**正确示例**

锚点使用 help，这是适用于所链接帮助资源的 rel 值。

```vue annotate="add:2"
<template>
<a href="/guide" rel="help">Guide</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_invalid_html_attribute.rs#L12) · [全部规则](all.md)

### `vue/no-lone-template`

禁止不必要的 `<template>` 元素

[错误示例](#vue-no-lone-template-bad) · [正确示例](#vue-no-lone-template-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-lone-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-lone-template-bad"></span>

**错误示例**

内层 template 没有指令或插槽作用，因此不具备结构上的用途。

```vue annotate="remove:2"
<template>
<div><template><p>Details</p></template></div>
</template>
```

<span id="vue-no-lone-template-good"></span>

**正确示例**

移除不必要的包装层后，段落直接位于 div 内。

```vue annotate="add:2"
<template>
<div><p>Details</p></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_lone_template.rs#L32) · [全部规则](all.md)

### `vue/no-multi-spaces`

禁止连续多个空格

[错误示例](#vue-no-multi-spaces-bad) · [正确示例](#vue-no-multi-spaces-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multi-spaces": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multi-spaces-bad"></span>

**错误示例**

属性之间，或元素名称与第一个属性之间，使用了两个空格。

```vue annotate="remove:2,3"
<template>
  <div  class="panel"></div>
  <div class="panel"  id="main"></div>
</template>
```

<span id="vue-no-multi-spaces-good"></span>

**正确示例**

同样的属性之间使用单个空格。

```vue annotate="add:2,3"
<template>
  <div class="panel"></div>
  <div class="panel" id="main"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multi_spaces.rs#L26) · [全部规则](all.md)

### `vue/no-multiple-objects-in-class`

禁止在 :class 数组绑定中使用多个对象字面量

[错误示例](#vue-no-multiple-objects-in-class-bad) · [正确示例](#vue-no-multiple-objects-in-class-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-objects-in-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-objects-in-class-bad"></span>

**错误示例**

class 数组包含两个可以合并的顶层对象字面量。

```vue annotate="remove:2,3"
<template>
<div :class="[{ a }, { b }]"></div>
<div :class="[{ active: isActive }, { error: hasError }]"></div>
</template>
```

<span id="vue-no-multiple-objects-in-class-good"></span>

**正确示例**

一个对象包含所有类名条件；只有一个对象和一个字符串的数组，以及包含非字面量项的数组，仍被允许。

```vue annotate="add:2,3,4"
<template>
<div :class="{ a, b }"></div>
<div :class="[{ active: isActive }, 'static']"></div>
<div :class="[foo, bar]"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_multiple_objects_in_class.rs#L33) · [全部规则](all.md)

### `vue/no-multiple-template-root`

禁止模板包含多个根节点

[错误示例](#vue-no-multiple-template-root-bad) · [正确示例](#vue-no-multiple-template-root-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

仅在需要单根节点约定时启用。Vue 3 通常支持片段。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-multiple-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-multiple-template-root-bad"></span>

**错误示例**

启用的单根节点规范在模板根部发现了两个同级段落。

```vue annotate="remove:2,3"
<template>
<p>First</p>
<p>Second</p>
</template>
```

<span id="vue-no-multiple-template-root-good"></span>

**正确示例**

使用 section 将段落包装为一个根节点；仅在需要单根节点约定时启用此规范。

```vue annotate="add:2"
<template>
<section><p>First</p><p>Second</p></section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_multiple_template_root.rs#L27) · [全部规则](all.md)

### `vue/no-mutating-props`

禁止修改组件 props

[错误示例](#vue-no-mutating-props-bad) · [正确示例](#vue-no-mutating-props-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-mutating-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-mutating-props-bad"></span>

**错误示例**

递增 props.count 会直接写入父组件提供的值。

```vue annotate="remove:4"
<script setup lang="ts">
const props = defineProps<{ count: number }>();

props.count++;
</script>
```

<span id="vue-no-mutating-props-good"></span>

**正确示例**

组件发出携带下一个值的 update:count 事件，由父组件负责更新 prop。

```vue annotate="add:3,5,6,7"
<script setup lang="ts">
const props = defineProps<{ count: number }>();
const emit = defineEmits<{ "update:count": [value: number] }>();

function increment() {
  emit("update:count", props.count + 1);
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_mutating_props.rs#L42) · [全部规则](all.md)

### `vue/no-negated-v-if-condition`

禁止在含有 v-else 的条件链中使用否定的 v-if 条件

[错误示例](#vue-no-negated-v-if-condition-bad) · [正确示例](#vue-no-negated-v-if-condition-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-negated-v-if-condition": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-negated-v-if-condition-bad"></span>

**错误示例**

成对的 v-if 和 v-else 分支以否定条件开头。

```vue
<template>
<div v-if="!ok">A</div>
<div v-else>B</div>
</template>
```

<span id="vue-no-negated-v-if-condition-good"></span>

**正确示例**

先使用肯定条件 ok；反转条件时，将原来的相反分支放在前面。单独使用否定的 v-if 和 !== 比较仍被允许。

```vue annotate="add:2,3,4,6,7"
<template>
<div v-if="ok">B</div>
<div v-else>A</div>

<div v-if="!ok">A</div>

<div v-if="a !== b">A</div>
<div v-else>B</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_negated_v_if_condition.rs#L37) · [全部规则](all.md)

### `vue/no-non-component-keep-alive-child`

禁止在 `<KeepAlive>` 的直属子层使用普通元素包装

[错误示例](#vue-no-non-component-keep-alive-child-bad) · [正确示例](#vue-no-non-component-keep-alive-child-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-non-component-keep-alive-child": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-non-component-keep-alive-child-bad"></span>

**错误示例**

KeepAlive 有条件地包装原生 div，而没有直接缓存 UserCard。

```vue annotate="remove:3"
<template>
  <KeepAlive>
    <div v-if="ready">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

<span id="vue-no-non-component-keep-alive-child-good"></span>

**正确示例**

第一个示例将 UserCard 设为条件子节点。v-show 包装层展示了不属于此条件子节点检查范围的结构，并不保证原生包装元素会被缓存。

```vue annotate="add:3,4,5,6"
<template>
  <KeepAlive>
    <UserCard v-if="ready" />
  </KeepAlive>
  <KeepAlive>
    <div v-show="opened">
      <UserCard />
    </div>
  </KeepAlive>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_non_component_keep_alive_child.rs#L14) · [全部规则](all.md)

### `vue/no-preprocessor-lang`

不建议使用 CSS 预处理器，优先使用现代 CSS

[错误示例](#vue-no-preprocessor-lang-bad) · [正确示例](#vue-no-preprocessor-lang-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 尚未在 SFC lint 中实现  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

当前支持情况: `no-sfc-finding`

此目录条目目前不会通过 SFC lint 报告该规则特有的问题。Bad/Good 对照描述的是预期规范，并非可执行的诊断。启用此 ID 也不会补上缺失的 SFC 检查。

**可配置的 ID（当前没有 SFC 诊断）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-preprocessor-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-preprocessor-lang-bad"></span>

**错误示例**

style 块通过 lang 选择 SCSS。这描述了不使用预处理器的预期规范；当前 SFC 路径不会报告此规则。

```vue annotate="remove:2"
<template><p>Notice</p></template>
<style lang="scss">
.notice { color: red; }
</style>
```

<span id="vue-no-preprocessor-lang-good"></span>

**正确示例**

相同的 CSS 声明省略了预处理器 lang。这修正了规范上的问题，但目前不会产生可执行的 Bad/Good 诊断差异。

```vue annotate="add:2"
<template><p>Notice</p></template>
<style>
.notice { color: red; }
</style>
```

正确示例展示预期规范；当前 SFC 流程不会为任一示例生成这条规则的诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_preprocessor_lang.rs#L22) · [全部规则](all.md)

### `vue/no-reserved-component-names`

禁止使用保留名称作为组件名称

[错误示例](#vue-no-reserved-component-names-bad) · [正确示例](#vue-no-reserved-component-names-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-reserved-component-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-reserved-component-names-bad"></span>

**错误示例**

组件名称 button 与原生 HTML 元素名称冲突。

```vue annotate="remove:1,2,3,4"
<script>
export default {
  name: "button",
};
</script>
```

<span id="vue-no-reserved-component-names-good"></span>

**正确示例**

AppButton 是应用组件名称，没有复用原生 button 名称。

```vue annotate="add:1,2,4,5,6,7,8,9"
<script setup lang="ts">
defineOptions({ name: "AppButton" });
</script>

<template>
  <Transition>
    <AppButton />
  </Transition>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_reserved_component_names.rs#L45) · [全部规则](all.md)

### `vue/no-root-v-if`

禁止在模板的唯一根元素上使用 v-if

[错误示例](#vue-no-root-v-if-bad) · [正确示例](#vue-no-root-v-if-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-root-v-if": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-root-v-if-bad"></span>

**错误示例**

组件根元素本身会在 v-if 控制下出现或消失。

```vue annotate="remove:2"
<template>
  <div v-if="show">content</div>
</template>
```

<span id="vue-no-root-v-if-good"></span>

**正确示例**

稳定的外层 div 始终作为根元素，而内层段落承担可见性条件。

```vue annotate="add:2,3,4"
<template>
  <div>
    <p v-if="show">content</p>
  </div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_root_v_if.rs#L40) · [全部规则](all.md)

### `vue/no-script-non-standard-lang`

不建议使用非标准的 script lang 值

[错误示例](#vue-no-script-non-standard-lang-bad) · [正确示例](#vue-no-script-non-standard-lang-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 尚未在 SFC lint 中实现  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

当前支持情况: `no-sfc-finding`

此目录条目目前不会通过 SFC lint 报告该规则特有的问题。Bad/Good 对照描述的是预期规范，并非可执行的诊断。启用此 ID 也不会补上缺失的 SFC 检查。

**可配置的 ID（当前没有 SFC 诊断）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-script-non-standard-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-script-non-standard-lang-bad"></span>

**错误示例**

脚本在 lang=coffee 下使用 CoffeeScript 语法。当前 SFC 路径不会针对该语言报告此目录规则。

```vue annotate="remove:1,2"
<script lang="coffee">
count = 0
</script>
<template><p>Notice</p></template>
```

<span id="vue-no-script-non-standard-lang-good"></span>

**正确示例**

脚本在 lang=ts 下使用普通 TypeScript 声明，展示预期的语言规范。

```vue annotate="add:1,2"
<script lang="ts">
const count = 0;
</script>
<template><p>Notice</p></template>
```

正确示例展示预期规范；当前 SFC 流程不会为任一示例生成这条规则的诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_script_non_standard_lang.rs#L44) · [全部规则](all.md)

### `vue/no-src-attribute`

不建议在 SFC 块上使用 src 属性

[错误示例](#vue-no-src-attribute-bad) · [正确示例](#vue-no-src-attribute-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-src-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-src-attribute-bad"></span>

**错误示例**

SFC 块通过 src 文件提供模板、脚本和样式内容。

```vue annotate="remove:1,2,3"
<template src="./template.html"></template>
<script src="./script.ts"></script>
<style src="./style.css"></style>
```

<span id="vue-no-src-attribute-good"></span>

**正确示例**

每个 SFC 块都包含自身内容，不使用外部 src 属性。

```vue annotate="add:1,2,3,4,5,6,7,8,9,10,11,12,13"
<template>
  <p>Hello</p>
</template>

<script setup lang="ts">
const label = "Hello";
</script>

<style scoped>
p {
  color: red;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_src_attribute.rs#L16) · [全部规则](all.md)

### `vue/no-static-inline-styles`

禁止静态内联 style 属性

[错误示例](#vue-no-static-inline-styles-bad) · [正确示例](#vue-no-static-inline-styles-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-static-inline-styles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-static-inline-styles-bad"></span>

**错误示例**

段落在 style 属性中包含固定的颜色声明。

```vue annotate="remove:1,2,3"
<template>
<p style="color: red">Notice</p>
</template>
```

<span id="vue-no-static-inline-styles-good"></span>

**正确示例**

notice 类和 scoped 样式表将固定颜色放在模板属性之外。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { color: red; }</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_static_inline_styles.rs#L15) · [全部规则](all.md)

### `vue/no-template-key`

禁止在 `<template>` 上使用 `key` 属性

[错误示例](#vue-no-template-key-bad) · [正确示例](#vue-no-template-key-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-key-bad"></span>

**错误示例**

非循环的 template 包装层具有 key，但它并不是需要 key 的迭代边界。

```vue annotate="remove:2"
<template>
<template :key="section"><div>Details</div></template>
</template>
```

<span id="vue-no-template-key-good"></span>

**正确示例**

key 属于 template v-for 迭代，用来标识每个重复的片段。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><div>{{ item.name }}</div></template>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_key.rs#L31) · [全部规则](all.md)

### `vue/no-template-lang`

不建议在 template 块上使用 lang 属性

[错误示例](#vue-no-template-lang-bad) · [正确示例](#vue-no-template-lang-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 尚未在 SFC lint 中实现  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

当前支持情况: `no-sfc-finding`

此目录条目目前不会通过 SFC lint 报告该规则特有的问题。Bad/Good 对照描述的是预期规范，并非可执行的诊断。启用此 ID 也不会补上缺失的 SFC 检查。

**可配置的 ID（当前没有 SFC 诊断）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-lang": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-lang-bad"></span>

**错误示例**

模板通过 lang 选择 Pug。这是只使用 HTML 的预期规范；当前 SFC 路径不会针对该目录 ID 产生诊断。

```vue annotate="remove:1,2"
<template lang="pug">
p Notice
</template>
```

<span id="vue-no-template-lang-good"></span>

**正确示例**

普通 HTML 模板省略 lang，直接使用段落。这展示了规范，并未声称当前 SFC 会报告问题。

```vue annotate="add:1,2"
<template>
<p>Notice</p>
</template>
```

正确示例展示预期规范；当前 SFC 流程不会为任一示例生成这条规则的诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_lang.rs#L38) · [全部规则](all.md)

### `vue/no-template-shadow`

禁止使用遮蔽外层作用域变量的变量名称

[错误示例](#vue-no-template-shadow-bad) · [正确示例](#vue-no-template-shadow-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

当前检查比较嵌套的 v-for 绑定。单个 v-for 绑定仅与脚本绑定同名时，不会因此被报告。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-shadow": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-shadow-bad"></span>

**错误示例**

内层 v-for 再次声明 item，在嵌套循环中遮蔽了外层 item 绑定。

```vue annotate="remove:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="item in item.children" :key="item.id">{{ item.name }}</span></div>
</template>
```

<span id="vue-no-template-shadow-good"></span>

**正确示例**

内层循环声明 child，外层行仍使用 item，内层行则使用 child。

```vue annotate="add:2"
<template>
<div v-for="item in items" :key="item.id"><span v-for="child in item.children" :key="child.id">{{ child.name }}</span></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_template_shadow.rs#L34) · [全部规则](all.md)

### `vue/no-template-target-blank`

禁止使用 target="_blank" 而不设置 rel="noopener noreferrer"

[错误示例](#vue-no-template-target-blank-bad) · [正确示例](#vue-no-template-target-blank-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-template-target-blank": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-template-target-blank-bad"></span>

**错误示例**

外部链接打开新的浏览上下文，却没有预期的 rel 保护。

```vue annotate="remove:2"
<template>
<a href="https://example.com" target="_blank">x</a>
</template>
```

<span id="vue-no-template-target-blank-good"></span>

**正确示例**

同一链接在 target=_blank 之外还包含 noopener noreferrer。

```vue annotate="add:2"
<template>
<a href="https://example.com" target="_blank" rel="noopener noreferrer">x</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_template_target_blank.rs#L33) · [全部规则](all.md)

### `vue/no-textarea-mustache`

禁止在 `<textarea>` 内使用双花括号插值

[错误示例](#vue-no-textarea-mustache-bad) · [正确示例](#vue-no-textarea-mustache-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-textarea-mustache": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-textarea-mustache-bad"></span>

**错误示例**

textarea 将 message 写在子内容插值中，而没有绑定其值。

```vue annotate="remove:2"
<template>
  <textarea>{{ message }}</textarea>
</template>
```

<span id="vue-no-textarea-mustache-good"></span>

**正确示例**

v-model 将可编辑的 textarea 值绑定到 message。

```vue annotate="add:2"
<template>
  <textarea v-model="message"></textarea>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_textarea_mustache.rs#L26) · [全部规则](all.md)

### `vue/no-undefined-refs`

禁止在模板中引用未定义的变量

[错误示例](#vue-no-undefined-refs-bad) · [正确示例](#vue-no-undefined-refs-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-undefined-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-undefined-refs-bad"></span>

**错误示例**

模板读取 missing，但脚本只声明了 message。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template>{{ missing }}</template>
```

<span id="vue-no-undefined-refs-good"></span>

**正确示例**

插值读取已存在的 message 绑定。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template>{{ message }}</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_undefined_refs.rs#L14) · [全部规则](all.md)

### `vue/no-unsafe-url`

警告可能不安全的 URL 绑定

[错误示例](#vue-no-unsafe-url-bad) · [正确示例](#vue-no-unsafe-url-good)

默认严重程度: `warning`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsafe-url": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsafe-url-bad"></span>

**错误示例**

锚点的目标地址以可执行的 javascript: 协议开头。

```vue annotate="remove:2"
<template>
<a href="javascript:alert(1)">Continue</a>
</template>
```

<span id="vue-no-unsafe-url-good"></span>

**正确示例**

锚点使用普通的本地导航目标 /next。

```vue annotate="add:2"
<template>
<a href="/next">Continue</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsafe_url.rs#L55) · [全部规则](all.md)

### `vue/no-unsandboxed-iframe`

要求 iframe 元素具有 sandbox 属性

[错误示例](#vue-no-unsandboxed-iframe-bad) · [正确示例](#vue-no-unsandboxed-iframe-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unsandboxed-iframe": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unsandboxed-iframe-bad"></span>

**错误示例**

嵌入的框架没有使用 sandbox 属性限制其能力。

```vue annotate="remove:2"
<template>
<iframe src="/embed"></iframe>
</template>
```

<span id="vue-no-unsandboxed-iframe-good"></span>

**正确示例**

sandbox 施加限制；需要脚本能力时，allow-scripts 显式允许这一项能力。

```vue annotate="add:2,3"
<template>
<iframe src="/embed" sandbox></iframe>
<iframe src="/embed" sandbox="allow-scripts"></iframe>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unsandboxed_iframe.rs#L32) · [全部规则](all.md)

### `vue/no-unused-components`

禁止注册未在模板中使用的组件

[错误示例](#vue-no-unused-components-bad) · [正确示例](#vue-no-unused-components-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-components-bad"></span>

**错误示例**

UserAvatar 作为组件导入，但模板从未渲染它。

```vue annotate="remove:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <p>{{ user.name }}</p>
</template>
```

<span id="vue-no-unused-components-good"></span>

**正确示例**

模板渲染导入的 UserAvatar，并传入 user 绑定。

```vue annotate="add:6"
<script setup lang="ts">
import UserAvatar from "./UserAvatar.vue";
</script>

<template>
  <UserAvatar :user="user" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_components.rs#L46) · [全部规则](all.md)

### `vue/no-unused-properties`

禁止 defineProps 中定义的未使用属性

[错误示例](#vue-no-unused-properties-bad) · [正确示例](#vue-no-unused-properties-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-properties-bad"></span>

**错误示例**

组件将 description 声明为 prop，但只渲染了 title。

```vue
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
</template>
```

<span id="vue-no-unused-properties-good"></span>

**正确示例**

模板引用了两个已声明的 props。

```vue annotate="add:7"
<script setup lang="ts">
defineProps<{ title: string; description: string }>();
</script>

<template>
  <h1>{{ title }}</h1>
  <p>{{ description }}</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_properties.rs#L94) · [全部规则](all.md)

### `vue/no-unused-refs`

报告从未在 &lt;script&gt; 中引用的模板 ref（ref="x"）

[错误示例](#vue-no-unused-refs-bad) · [正确示例](#vue-no-unused-refs-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-refs-bad"></span>

**错误示例**

模板声明了名为 unused 的 ref，但脚本中没有对应的引用绑定。

```vue annotate="remove:1,3"
<template><input ref="unused" /></template>
<script setup>
const x = 1
</script>
```

<span id="vue-no-unused-refs-good"></span>

**正确示例**

模板 ref inputEl 在 script setup 中具有同名的 ref 绑定。

```vue annotate="add:1,3,4"
<template><input ref="inputEl" /></template>
<script setup>
import { ref } from 'vue'
const inputEl = ref(null)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_unused_refs.rs#L60) · [全部规则](all.md)

### `vue/no-unused-setup-bindings`

禁止从未读取的 script setup 绑定

[错误示例](#vue-no-unused-setup-bindings-bad) · [正确示例](#vue-no-unused-setup-bindings-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-setup-bindings": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-setup-bindings-bad"></span>

**错误示例**

script setup 中的 message 绑定从未被模板读取。

```vue annotate="remove:2"
<script setup>const message = "Hello";</script>
<template><p>Welcome</p></template>
```

<span id="vue-no-unused-setup-bindings-good"></span>

**正确示例**

段落插值使用 message，从而使用了声明的绑定。

```vue annotate="add:2"
<script setup>const message = "Hello";</script>
<template><p>{{ message }}</p></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/facts/unused_setup_bindings.rs#L19) · [全部规则](all.md)

### `vue/no-unused-vars`

禁止 v-for 和 v-slot 指令中未使用的变量定义

[错误示例](#vue-no-unused-vars-bad) · [正确示例](#vue-no-unused-vars-good)

默认严重程度: `warning`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-unused-vars": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-unused-vars-bad"></span>

**错误示例**

循环声明了未使用的 index，插槽也声明了未被引用的 foo。

```vue annotate="remove:2,3,4"
<template>
  <li v-for="(item, index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ foo }">
    <span>Hello</span>
  </template>
</template>
```

<span id="vue-no-unused-vars-good"></span>

**正确示例**

示例使用 index，或将其命名为 _index 以标记有意不使用；插槽则渲染 data。这里的索引 key 只展示变量使用，并不建议用它保持列表项身份稳定。

```vue annotate="add:2,3,4,5"
<template>
  <li v-for="(item, index) in items" :key="index">{{ item.name }}</li>
  <li v-for="(item, _index) in items" :key="item.id">{{ item.name }}</li>
  <template v-slot="{ data }">
    <span>{{ data }}</span>
  </template>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_unused_vars.rs#L48) · [全部规则](all.md)

### `vue/no-use-v-else-with-v-for`

禁止在同一元素上使用 `v-else-if` 或 `v-else` 与 `v-for`

[错误示例](#vue-no-use-v-else-with-v-for-bad) · [正确示例](#vue-no-use-v-else-with-v-for-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-use-v-else-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-else-with-v-for-bad"></span>

**错误示例**

else 分支和 v-for 迭代都附加在同一段落上。

```vue annotate="remove:3"
<template>
<p v-if="ready">Ready</p>
<p v-else v-for="item in items" :key="item.id">{{ item.name }}</p>
</template>
```

<span id="vue-no-use-v-else-with-v-for-good"></span>

**正确示例**

独立的 template 承担 v-else，其子段落承担 v-for。

```vue annotate="add:3"
<template>
<p v-if="ready">Ready</p>
<template v-else><p v-for="item in items" :key="item.id">{{ item.name }}</p></template>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_else_with_v_for.rs#L19) · [全部规则](all.md)

### `vue/no-use-v-if-with-v-for`

禁止在同一元素上使用 `v-if` 与 `v-for`

[错误示例](#vue-no-use-v-if-with-v-for-bad) · [正确示例](#vue-no-use-v-if-with-v-for-good)

默认严重程度: `warning`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-use-v-if-with-v-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-use-v-if-with-v-for-bad"></span>

**错误示例**

同一个列表元素同时使用 v-if 和 v-for，并通过循环绑定检查可见性。

```vue annotate="remove:2"
<template>
  <li v-for="item in items" v-if="item.visible" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

<span id="vue-no-use-v-if-with-v-for-good"></span>

**正确示例**

计算属性中的集合先筛选出可见项，再由模板进行迭代。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const visibleItems = computed(() => items.filter((item) => item.visible));
</script>

<template>
  <li v-for="item in visibleItems" :key="item.id">
    {{ item.name }}
  </li>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_use_v_if_with_v_for.rs#L35) · [全部规则](all.md)

### `vue/no-useless-mustaches`

禁止表达式为常量字符串字面量的双花括号插值

[错误示例](#vue-no-useless-mustaches-bad) · [正确示例](#vue-no-useless-mustaches-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-mustaches": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-mustaches-bad"></span>

**错误示例**

插值仅包含常量字符串，无需对表达式求值。

```vue annotate="remove:2,3,4"
<template>
<div>{{ 'x' }}</div>
<div>{{ "x" }}</div>
<div>{{ `x` }}</div>
</template>
```

<span id="vue-no-useless-mustaches-good"></span>

**正确示例**

字面文本直接写入模板；变量表达式、带插值的模板字符串，以及有意作为分隔符的空白，仍可使用插值。

```vue annotate="add:2,3,4,5"
<template>
<div>x</div>
<div>{{ x }}</div>
<div>{{ `pre-${x}` }}</div>
<span>A</span> {{ " " }} <span>B</span>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_mustaches.rs#L37) · [全部规则](all.md)

### `vue/no-useless-template-attributes`

禁止在 `<template>` 元素上使用不起作用的属性

[错误示例](#vue-no-useless-template-attributes-bad) · [正确示例](#vue-no-useless-template-attributes-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-template-attributes": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-template-attributes-bad"></span>

**错误示例**

条件 template 具有 class，但这个结构性包装层不会渲染可接收该属性的 DOM 元素。

```vue annotate="remove:2"
<template>
<section><template v-if="ready" class="notice"><p>Ready</p></template></section>
</template>
```

<span id="vue-no-useless-template-attributes-good"></span>

**正确示例**

class 移到实际渲染的段落上，v-if 则保留在结构性 template 上。

```vue annotate="add:2"
<template>
<section><template v-if="ready"><p class="notice">Ready</p></template></section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_useless_template_attributes.rs#L32) · [全部规则](all.md)

### `vue/no-useless-v-bind`

禁止值为普通字符串字面量的 v-bind

[错误示例](#vue-no-useless-v-bind-bad) · [正确示例](#vue-no-useless-v-bind-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-useless-v-bind": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-useless-v-bind-bad"></span>

**错误示例**

foo 绑定对带引号的常量字符串，或不含插值的模板字符串，进行求值。

```vue annotate="remove:2,3"
<template>
<div :foo="'bar'"></div>
<div :foo="`bar`"></div>
</template>
```

<span id="vue-no-useless-v-bind-good"></span>

**正确示例**

常量值改为静态属性；变量值和含插值的值保留绑定。

```vue annotate="add:2,3,4"
<template>
<div foo="bar"></div>
<div :foo="bar"></div>
<div :foo="`pre-${bar}`"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_useless_v_bind.rs#L29) · [全部规则](all.md)

### `vue/no-v-for-template-key-on-child`

禁止在 `<template v-for>` 的子节点上使用 `key`

[错误示例](#vue-no-v-for-template-key-on-child-bad) · [正确示例](#vue-no-v-for-template-key-on-child-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-for-template-key-on-child": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-for-template-key-on-child-bad"></span>

**错误示例**

子段落具有 key，但 template 迭代自身没有 key。

```vue annotate="remove:2"
<template>
<template v-for="item in items"><p :key="item.id">{{ item.name }}</p></template>
</template>
```

<span id="vue-no-v-for-template-key-on-child-good"></span>

**正确示例**

key 移到 template v-for 上，用来标识整个重复片段。

```vue annotate="add:2"
<template>
<template v-for="item in items" :key="item.id"><p>{{ item.name }}</p></template>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_for_template_key_on_child.rs#L30) · [全部规则](all.md)

### `vue/no-v-html`

警告使用 v-html，以避免 XSS 漏洞

[错误示例](#vue-no-v-html-bad) · [正确示例](#vue-no-v-html-good)

默认严重程度: `warning`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-html": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-html-bad"></span>

**错误示例**

v-html 将 content 解释为 HTML，而不是普通文本。

```vue annotate="remove:2"
<template>
  <article v-html="content" />
</template>
```

<span id="vue-no-v-html-good"></span>

**正确示例**

双花括号插值将 content 显示为转义后的文本，而不注入 HTML。

```vue annotate="add:2"
<template>
  <article>{{ content }}</article>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_html.rs#L51) · [全部规则](all.md)

### `vue/no-v-text`

禁止 v-text 指令，优先使用双花括号插值

[错误示例](#vue-no-v-text-bad) · [正确示例](#vue-no-v-text-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-bad"></span>

**错误示例**

div 的内容通过 v-text 指令提供。

```vue annotate="remove:2"
<template>
<div v-text="message"></div>
</template>
```

<span id="vue-no-v-text-good"></span>

**正确示例**

双花括号插值直接在元素内容中表达相同的文本绑定。

```vue annotate="add:2"
<template>
<div>{{ message }}</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/no_v_text.rs#L31) · [全部规则](all.md)

### `vue/no-v-text-v-html-on-component`

禁止在组件元素上使用 v-text / v-html

[错误示例](#vue-no-v-text-v-html-on-component-bad) · [正确示例](#vue-no-v-text-v-html-on-component-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/no-v-text-v-html-on-component": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-no-v-text-v-html-on-component-bad"></span>

**错误示例**

组件标签使用 v-html 或 v-text，这些指令会替换元素内容，而不是提供组件插槽。

```vue annotate="remove:2,3"
<template>
  <MyComponent v-html="content" />
  <MyComponent v-text="content" />
</template>
```

<span id="vue-no-v-text-v-html-on-component-good"></span>

**正确示例**

原生 HTML 目标可接收这些指令；MyComponent 通过默认插槽接收内容。

```vue annotate="add:2,3,4"
<template>
  <div v-html="content"></div>
  <component is="div" v-html="content" />
  <MyComponent>{{ content }}</MyComponent>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/no_v_text_v_html_on_component.rs#L33) · [全部规则](all.md)

### `vue/permitted-contents`

要求符合 HTML 内容模型规则

[错误示例](#vue-permitted-contents-bad) · [正确示例](#vue-permitted-contents-good)

默认严重程度: `error`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/permitted-contents": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-permitted-contents-bad"></span>

**错误示例**

示例在 p 中放置块级内容、省略表格主体、嵌套交互控件，或将 div 直接放在 ul 内。

```vue annotate="remove:2,3,4,5"
<template>
  <p><div>block in a paragraph</div></p>
  <table><tr><td>row without tbody</td></tr></table>
  <a href="#"><button type="button">nested control</button></a>
  <ul><div>not a list item</div></ul>
</template>
```

<span id="vue-permitted-contents-good"></span>

**正确示例**

示例使用段落内的行内内容、显式 tbody 和 li 子节点。自定义 MyItem 不被视为已知的原生 ul 子节点。

```vue annotate="add:2,3,4"
<template>
  <p><span>inline in a paragraph</span></p>
  <table><tbody><tr><td>cell</td></tr></tbody></table>
  <ul><li>list item</li><MyItem /></ul>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/permitted_contents.rs#L56) · [全部规则](all.md)

### `vue/prefer-props-shorthand`

建议使用 props 简写语法（Vue 3.4+）

[错误示例](#vue-prefer-props-shorthand-bad) · [正确示例](#vue-prefer-props-shorthand-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-props-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-props-shorthand-bad"></span>

**错误示例**

每个绑定都重复了对应的变量名，包括与连字符参数对应的驼峰名称。

```vue annotate="remove:2,3,4,5"
<template>
  <MyComponent :foo="foo" />
  <MyComponent :user-name="userName" />
  <span :style="style" />
  <div :aria-label="ariaLabel" />
</template>
```

<span id="vue-prefer-props-shorthand-good"></span>

**正确示例**

Vue 3.4+ 的同名绑定简写省略了重复表达式；bar 等不同的来源变量仍显式写出。

```vue annotate="add:2,3,4,5,6"
<template>
  <MyComponent :foo />
  <MyComponent :user-name />
  <span :style />
  <div :aria-label />
  <MyComponent :foo="bar" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_props_shorthand.rs#L39) · [全部规则](all.md)

### `vue/prefer-true-attribute-shorthand`

优先使用绑定到 `true` 的布尔属性简写

[错误示例](#vue-prefer-true-attribute-shorthand-bad) · [正确示例](#vue-prefer-true-attribute-shorthand-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prefer-true-attribute-shorthand": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

**错误示例**

原生布尔属性 disabled 绑定了常量 true。

```vue annotate="remove:2"
<template>
<input :disabled="true" />
</template>
```

<span id="vue-prefer-true-attribute-shorthand-good"></span>

**正确示例**

原生属性使用布尔简写。false 绑定和组件 props 保留显式值。

```vue annotate="add:2,3,4,5"
<template>
<input disabled />
<input :disabled="false" />
<MyComponent :visible="true" />
<MyComponent :visible="isVisible" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/prefer_true_attribute_shorthand.rs#L38) · [全部规则](all.md)

### `vue/prop-name-casing`

规范所声明 prop 名称的大小写风格

[错误示例](#vue-prop-name-casing-bad) · [正确示例](#vue-prop-name-casing-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

检查所声明的 prop 名称，而不是传给子组件的属性的大小写风格。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/prop-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-prop-name-casing-bad"></span>

**错误示例**

声明的 prop 名称 user_name 使用下划线分隔。

```vue annotate="remove:2,4"
<script setup lang="ts">
defineProps<{ user_name: string }>();
</script>
<template><p>{{ user_name }}</p></template>
```

<span id="vue-prop-name-casing-good"></span>

**正确示例**

声明及其模板引用都使用驼峰名称 userName。

```vue annotate="add:2,4"
<script setup lang="ts">
defineProps<{ userName: string }>();
</script>
<template><p>{{ userName }}</p></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/prop_name_casing.rs#L50) · [全部规则](all.md)

### `vue/require-component-is`

要求 `<component>` 元素具有 `v-bind:is`

[错误示例](#vue-require-component-is-bad) · [正确示例](#vue-require-component-is-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-is": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-is-bad"></span>

**错误示例**

动态 `<component>` 没有 `is` 目标，因此 Vue 无法选择要渲染的组件。

```vue annotate="remove:2"
<template>
  <component />
</template>
```

<span id="vue-require-component-is-good"></span>

**正确示例**

`:is="currentComponent"` 提供组件选择；该绑定可在运行时变化。

```vue annotate="add:2"
<template>
  <component :is="currentComponent" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_component_is.rs#L27) · [全部规则](all.md)

### `vue/require-component-registration`

要求显式导入或注册组件

[错误示例](#vue-require-component-registration-bad) · [正确示例](#vue-require-component-registration-good)

默认严重程度: `warning`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

列出由应用插件或 Musea previewSetup 提供的明确组件名称。接受 PascalCase 和 kebab-case 写法；不解析正则表达式。设置选项不会启用规则。后面的配置层会替换列表；空列表会清除继承的名称。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-component-registration": "warn"
      },
      "ruleOptions": {
        "vue/require-component-registration": {
          "globals": [
            "MyButton",
            "MyIcon"
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-component-registration-bad"></span>

**错误示例**

`MissingWidget` 既未注册，也不在配置的全局组件允许列表中。

```vue annotate="remove:2"
<template>
<MissingWidget />
</template>
```

<span id="vue-require-component-registration-good"></span>

**正确示例**

`MyButton` 列在示例的 `globals` 选项中。该选项让已知的全局组件免于此检查，并不会注册或导入组件。

```vue annotate="add:2"
<template>
<MyButton />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/require_component_registration.rs#L56) · [全部规则](all.md)

### `vue/require-scoped-style`

要求 style 标签具有 scoped 属性

[错误示例](#vue-require-scoped-style-bad) · [正确示例](#vue-require-scoped-style-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-scoped-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-scoped-style-bad"></span>

**错误示例**

`.button` 样式没有作用域限制，可能影响组件外部匹配的元素。

```vue annotate="remove:1"
<style>
.button {
  color: red;
}
</style>
```

<span id="vue-require-scoped-style-good"></span>

**正确示例**

添加 `scoped`，为同样的选择器和声明应用 Vue 的组件作用域。

```vue annotate="add:1"
<style scoped>
.button {
  color: red;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_scoped_style.rs#L49) · [全部规则](all.md)

### `vue/require-toggle-inside-transition`

要求 `<transition>` 包裹的元素具有切换条件

[错误示例](#vue-require-toggle-inside-transition-bad) · [正确示例](#vue-require-toggle-inside-transition-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-toggle-inside-transition": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-toggle-inside-transition-bad"></span>

**错误示例**

`<Transition>` 内的静态子节点没有条件可见性或动态选择，无法触发进入或离开的变化。

```vue annotate="remove:3"
<template>
<transition>
  <div>content</div>
</transition>
</template>
```

<span id="vue-require-toggle-inside-transition-good"></span>

**正确示例**

`v-if="show"` 改变子节点是否存在，为过渡提供进入和离开的边界。

```vue annotate="add:3"
<template>
<transition>
  <div v-if="show">content</div>
</transition>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_toggle_inside_transition.rs#L48) · [全部规则](all.md)

### `vue/require-v-for-key`

要求 `v-for` 指令搭配 `v-bind:key`

[错误示例](#vue-require-v-for-key-bad) · [正确示例](#vue-require-v-for-key-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/require-v-for-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-require-v-for-key-bad"></span>

**错误示例**

每个重复的 `<li>` 都缺少在列表更新时标识对应列表项的 key。

```vue annotate="remove:2"
<template>
  <li v-for="item in items">{{ item.name }}</li>
</template>
```

<span id="vue-require-v-for-key-good"></span>

**正确示例**

`:key="item.id"` 为每个重复节点提供列表项的身份，而不是当前位置。

```vue annotate="add:2"
<template>
  <li v-for="item in items" :key="item.id">{{ item.name }}</li>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/require_v_for_key.rs#L35) · [全部规则](all.md)

### `vue/scoped-event-names`

建议使用 context:event 格式的作用域事件名称

[错误示例](#vue-scoped-event-names-bad) · [正确示例](#vue-scoped-event-names-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/scoped-event-names": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-scoped-event-names-bad"></span>

**错误示例**

`playAudio`、`pauseAudio` 和 `reloadAudio` 使用驼峰后缀表达作用域，不符合规则要求的冒号分隔事件规范。

```vue annotate="remove:3,4,5"
<template>
  <AudioPlayer
    @playAudio="play"
    @pauseAudio="pause"
    @reloadAudio="reload"
  />
</template>
```

<span id="vue-scoped-event-names-good"></span>

**正确示例**

`audio:play`、`audio:pause` 和 `audio:reload` 共享显式的 `audio:` 作用域。发出事件的组件必须使用相同名称。

```vue annotate="add:3,4,5"
<template>
  <AudioPlayer
    @audio:play="play"
    @audio:pause="pause"
    @audio:reload="reload"
  />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/scoped_event_names.rs#L30) · [全部规则](all.md)

### `vue/sfc-element-order`

要求 SFC 顶层元素保持一致的顺序

[错误示例](#vue-sfc-element-order-bad) · [正确示例](#vue-sfc-element-order-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/sfc-element-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-sfc-element-order-bad"></span>

**错误示例**

style 块位于 script 块之前，不符合配置的 SFC 块顺序。

```vue annotate="remove:2,6,7,8"
<style scoped>
.panel {
  color: red;
}
</style>
<script setup lang="ts">
const label = "Save";
</script>
```

<span id="vue-sfc-element-order-good"></span>

**正确示例**

各块按照 script → template → style 排列。项目可通过此规则的类型化选项选择其他顺序。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts">
const label = "Save";
</script>

<template>
  <p>{{ label }}</p>
</template>

<style scoped>
p {
  color: red;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/sfc_element_order.rs#L50) · [全部规则](all.md)

### `vue/single-style-block`

建议只使用一个 style 块

[错误示例](#vue-single-style-block-bad) · [正确示例](#vue-single-style-block-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/single-style-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-single-style-block-bad"></span>

**错误示例**

组件将带作用域的 panel 和 title 样式分散在两个 style 块中。

```vue annotate="remove:5,6,7"
<style scoped>
.panel {
  color: red;
}
</style>

<style scoped>
.title {
  color: blue;
}
</style>
```

<span id="vue-single-style-block-good"></span>

**正确示例**

两个选择器都保留在同一个带作用域的 style 块中，满足单块规范，也没有丢弃任何样式。

```vue
<style scoped>
.panel {
  color: red;
}
.title {
  color: blue;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/single_style_block.rs#L41) · [全部规则](all.md)

### `vue/slot-name-casing`

要求通过 v-slot 使用的具名插槽采用 kebab-case

[错误示例](#vue-slot-name-casing-bad) · [正确示例](#vue-slot-name-casing-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/slot-name-casing": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-slot-name-casing-bad"></span>

**错误示例**

具名插槽 `mySlot` 使用驼峰命名，而规则要求连字符名称。

```vue annotate="remove:2"
<template>
<MyCard><template #mySlot>Content</template></MyCard>
</template>
```

<span id="vue-slot-name-casing-good"></span>

**正确示例**

`#my-slot` 使用 kebab-case。将对应的插槽出口重命名为相同名称。

```vue annotate="add:2"
<template>
<MyCard><template #my-slot>Content</template></MyCard>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/slot_name_casing.rs#L34) · [全部规则](all.md)

### `vue/this-in-template`

禁止在模板表达式中使用 `this.`

[错误示例](#vue-this-in-template-bad) · [正确示例](#vue-this-in-template-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/this-in-template": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-this-in-template-bad"></span>

**错误示例**

模板表达式显式访问 `this.message`、`this.className` 和 `this.handleClick`，但 Vue 已直接暴露这些绑定。

```vue annotate="remove:2,3,4"
<template>
<div>{{ this.message }}</div>
<div :class="this.className"></div>
<button @click="this.handleClick()"></button>
</template>
```

<span id="vue-this-in-template-good"></span>

**正确示例**

直接使用 `message`、`className` 和 `handleClick`。字面字符串 `'this.is.a.string'` 保持不变，因为它不是成员访问。

```vue annotate="add:2,3,4,5"
<template>
<div>{{ message }}</div>
<div :class="className"></div>
<button @click="handleClick()"></button>
<div>{{ 'this.is.a.string' }}</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/this_in_template.rs#L33) · [全部规则](all.md)

### `vue/use-unique-element-ids`

要求使用 useId() 生成唯一元素 ID，而不是静态字面量

[错误示例](#vue-use-unique-element-ids-bad) · [正确示例](#vue-use-unique-element-ids-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-unique-element-ids": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-unique-element-ids-bad"></span>

**错误示例**

组件的每个实例都会复用字面量 ID `email`，多个实例同时渲染时，标签可能指向错误的输入框。

```vue annotate="remove:2,3"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

<span id="vue-use-unique-element-ids-good"></span>

**正确示例**

`useId()` 生成实例的 `emailId`；将同一值绑定到标签的 `for` 和输入框的 `id`。

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup>
import { useId } from "vue";

const emailId = useId();
</script>

<template>
  <label :for="emailId">Email</label>
  <input :id="emailId" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_unique_element_ids.rs#L52) · [全部规则](all.md)

### `vue/use-v-on-exact`

存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`

[错误示例](#vue-use-v-on-exact-bad) · [正确示例](#vue-use-v-on-exact-good)

默认严重程度: `warning`  
预设: `essential`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/use-v-on-exact": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-use-v-on-exact-bad"></span>

**错误示例**

普通点击处理器也可能在 Ctrl-click 时运行，与独立的 `.ctrl` 处理器重叠。

```vue annotate="remove:2"
<template>
  <button type="button" @click="handleClick" @click.ctrl="handleCtrlClick">
    Save
  </button>
</template>
```

<span id="vue-use-v-on-exact-good"></span>

**正确示例**

`.exact` 将普通点击处理器限制为未按修饰键的点击；Ctrl 专用处理器仍保持独立。

```vue annotate="add:2,3,4,5,6"
<template>
  <button
    type="button"
    @click.exact="handleClick"
    @click.ctrl="handleCtrlClick"
  >
    Save
  </button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/use_v_on_exact.rs#L28) · [全部规则](all.md)

### `vue/v-bind-style`

规范 `v-bind` 指令风格

[错误示例](#vue-v-bind-style-bad) · [正确示例](#vue-v-bind-style-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-bind-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-bind-style-bad"></span>

**错误示例**

`v-bind:class` 使用了完整形式，但配置的绑定风格要求冒号简写。

```vue annotate="remove:2"
<template>
  <div v-bind:class="panelClass"></div>
</template>
```

<span id="vue-v-bind-style-good"></span>

**正确示例**

`:class` 使用所需的简写，并保留相同表达式；此规则检查写法，而非值的类型。

```vue annotate="add:2"
<template>
  <div :class="panelClass"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_bind_style.rs#L30) · [全部规则](all.md)

### `vue/v-on-event-hyphenation`

要求组件上 v-on 中的自定义事件名称使用连字符

[错误示例](#vue-v-on-event-hyphenation-bad) · [正确示例](#vue-v-on-event-hyphenation-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-event-hyphenation": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-event-hyphenation-bad"></span>

**错误示例**

自定义组件监听器使用了 `@myEvent`，而不是连字符事件名称。

```vue annotate="remove:2,3"
<template>
<MyComponent @myEvent="handler" />
<MyComponent v-on:myEvent="handler" />
</template>
```

<span id="vue-v-on-event-hyphenation-good"></span>

**正确示例**

`@my-event` 符合要求的自定义事件写法。下方展示的原生元素监听器和动态事件参数不在此检查范围内。

```vue annotate="add:2,3,4"
<template>
<MyComponent @my-event="handler" />
<div @myEvent="handler" />
<MyComponent @[dynamicEvent]="handler" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_event_hyphenation.rs#L35) · [全部规则](all.md)

### `vue/v-on-handler-style`

要求 v-on 处理器写为方法引用或内联函数

[错误示例](#vue-v-on-handler-style-bad) · [正确示例](#vue-v-on-handler-style-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-handler-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-handler-style-bad"></span>

**错误示例**

处理器将修改操作和多条语句直接写在事件属性中。

```vue annotate="remove:2,3,4"
<template>
<button @click="count++"></button>
<button @click="doThis(); doThat()"></button>
<button @click="foo = bar"></button>
</template>
```

<span id="vue-v-on-handler-style-good"></span>

**正确示例**

使用处理器引用；需要内联逻辑时，使用箭头函数或函数表达式。函数边界使处理器形式明确。

```vue annotate="add:2,3,4,5"
<template>
<button @click="handler"></button>
<button @click="foo.bar"></button>
<button @click="() => count++"></button>
<button @click="function () { count++ }"></button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/v_on_handler_style.rs#L33) · [全部规则](all.md)

### `vue/v-on-style`

规范 `v-on` 指令风格

[错误示例](#vue-v-on-style-bad) · [正确示例](#vue-v-on-style-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-on-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-on-style-bad"></span>

**错误示例**

`v-on:click` 使用完整的事件监听形式，但规则要求简写。

```vue annotate="remove:2"
<template>
  <div v-on:click="handleClick"></div>
</template>
```

<span id="vue-v-on-style-good"></span>

**正确示例**

`@click` 使用配置的简写，并保留同一处理器。

```vue annotate="add:2"
<template>
  <div @click="handleClick"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_on_style.rs#L28) · [全部规则](all.md)

### `vue/v-slot-style`

规范 `v-slot` 指令风格

[错误示例](#vue-v-slot-style-bad) · [正确示例](#vue-v-slot-style-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/v-slot-style": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-v-slot-style-bad"></span>

**错误示例**

组件使用 `#default`，template 使用 `v-slot:header`，与规则针对各上下文要求的风格相反。

```vue annotate="remove:2,4"
<template>
  <MyComponent #default="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template v-slot:header>Header</template>
  </MyComponent>
</template>
```

<span id="vue-v-slot-style-good"></span>

**正确示例**

组件的默认插槽使用 `v-slot`，template 的具名插槽使用 `#header`。

```vue annotate="add:2,4"
<template>
  <MyComponent v-slot="props">{{ props.item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/v_slot_style.rs#L41) · [全部规则](all.md)

### `vue/valid-attribute-name`

要求属性名称有效

[错误示例](#vue-valid-attribute-name-bad) · [正确示例](#vue-valid-attribute-name-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

错误示例的诊断: `parser/template`

格式错误的属性写法会先由 parser/template 诊断，之后该防御性规则才能读取属性。因此 Bad 报告的是 parser/template，并不保证另有独立的 vue/valid-attribute-name 问题。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-attribute-name": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-attribute-name-bad"></span>

**错误示例**

`my"attr` 内的引号使属性名称格式错误。此示例产生解析器的 `parser/template` 诊断，并不保证另有独立的规则诊断。

```vue annotate="remove:2"
<template>
<div my"attr="value"></div>
</template>
```

<span id="vue-valid-attribute-name-good"></span>

**正确示例**

`my-attr` 是格式正确的属性名称，因此模板解析器可以读取该属性及其值。

```vue annotate="add:2"
<template>
<div my-attr="value"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_attribute_name.rs#L27) · [全部规则](all.md)

### `vue/valid-template-root`

要求 `<template>` 根节点符合 Vue 3 片段语义

[错误示例](#vue-valid-template-root-bad) · [正确示例](#vue-valid-template-root-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-template-root": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-template-root-bad"></span>

**错误示例**

普通的嵌套 `<template>` 占据模板根部，却没有赋予它渲染作用的指令。

```vue annotate="remove:2"
<template>
  <template>content</template>
</template>
```

<span id="vue-valid-template-root-good"></span>

**正确示例**

`<div>` 是可渲染的根元素。此示例并未对 Vue 3 片段施加普遍的单根节点限制。

```vue annotate="add:2"
<template>
  <div>content</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_template_root.rs#L82) · [全部规则](all.md)

### `vue/valid-v-bind`

要求 `v-bind` 指令有效

[错误示例](#vue-valid-v-bind-bad) · [正确示例](#vue-valid-v-bind-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-bind": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-bind-bad"></span>

**错误示例**

不带参数的 `v-bind` 缺少对象表达式，空参数形式则缺少属性名称。

```vue annotate="remove:2,3"
<template>
  <div v-bind></div>
  <div :></div>
</template>
```

<span id="vue-valid-v-bind-good"></span>

**正确示例**

提供属性与表达式、绑定对象，或使用 Vue 3.4+ 的同名简写，例如 `:loading`。

```vue annotate="add:2,3,4"
<template>
  <div :class="panelClass"></div>
  <div v-bind="{ class: panelClass }"></div>
  <div :loading></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_bind.rs#L30) · [全部规则](all.md)

### `vue/valid-v-cloak`

要求 `v-cloak` 指令有效

[错误示例](#vue-valid-v-cloak-bad) · [正确示例](#vue-valid-v-cloak-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-cloak": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-cloak-bad"></span>

**错误示例**

`v-cloak` 设置了值、参数或修饰符，但它不接受这些内容。

```vue annotate="remove:2,3,4"
<template>
<div v-cloak="foo"></div>
<div v-cloak:arg></div>
<div v-cloak.mod></div>
</template>
```

<span id="vue-valid-v-cloak-good"></span>

**正确示例**

使用不带值的 `v-cloak`；CSS 可隐藏元素，直到 Vue 挂载后移除此属性。

```vue annotate="add:2"
<template>
<div v-cloak></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_cloak.rs#L27) · [全部规则](all.md)

### `vue/valid-v-else`

要求 `v-else` 指令有效

[错误示例](#vue-valid-v-else-bad) · [正确示例](#vue-valid-v-else-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-else": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-else-bad"></span>

**错误示例**

示例为 `v-else` 提供表达式、将其与 `v-if` 组合，或省略了紧邻其前的条件分支。

```vue annotate="remove:2,3"
<template>
  <div v-else="ready"></div>
  <div v-else v-if="ready"></div>
  <div v-else></div>
</template>
```

<span id="vue-valid-v-else-good"></span>

**正确示例**

将不带值的 `v-else` 紧接在对应的 `v-if` 分支之后。

```vue annotate="add:2"
<template>
  <div v-if="ready"></div>
  <div v-else></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_else.rs#L32) · [全部规则](all.md)

### `vue/valid-v-for`

要求 `v-for` 指令有效

[错误示例](#vue-valid-v-for-bad) · [正确示例](#vue-valid-v-for-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-for": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-for-bad"></span>

**错误示例**

循环省略了迭代表达式，或添加了不受支持的 `.stop` 修饰符。

```vue annotate="remove:2,3,4"
<template>
  <div v-for></div>
  <div v-for=""></div>
  <div v-for.stop="item in items"></div>
</template>
```

<span id="vue-valid-v-for-good"></span>

**正确示例**

使用 `item in items` 或 `(item, index) of items`，提供完整的迭代表达式及示例中的 key。

```vue annotate="add:2,3"
<template>
  <div v-for="item in items" :key="item.id"></div>
  <div v-for="(item, index) of items" :key="index"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_for.rs#L31) · [全部规则](all.md)

### `vue/valid-v-html`

要求 `v-html` 指令有效

[错误示例](#vue-valid-v-html-bad) · [正确示例](#vue-valid-v-html-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-html": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-html-bad"></span>

**错误示例**

`v-html` 缺少表达式，或使用了此指令不支持的参数或修饰符。

```vue annotate="remove:2,3,4"
<template>
<div v-html></div>
<div v-html:arg="foo"></div>
<div v-html.mod="foo"></div>
</template>
```

<span id="vue-valid-v-html-good"></span>

**正确示例**

`v-html="html"` 提供有效表达式。语法有效并不会清理 HTML，也不会让不可信内容变得安全。

```vue annotate="add:2"
<template>
<div v-html="html"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_html.rs#L28) · [全部规则](all.md)

### `vue/valid-v-if`

要求 `v-if` 指令有效

[错误示例](#vue-valid-v-if-bad) · [正确示例](#vue-valid-v-if-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-if": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-if-bad"></span>

**错误示例**

条件省略了表达式，或在同一节点上将 `v-if` 与 else 指令组合。

```vue annotate="remove:2,3,4"
<template>
  <div v-if></div>
  <div v-if=""></div>
  <div v-if="ready" v-else></div>
</template>
```

<span id="vue-valid-v-if-good"></span>

**正确示例**

每个 `v-if` 都具有 `ready` 或 `count > 0` 等非空条件，且没有不兼容的 else 指令。

```vue annotate="add:2,3"
<template>
  <div v-if="ready"></div>
  <div v-if="count > 0"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_if.rs#L29) · [全部规则](all.md)

### `vue/valid-v-memo`

要求 `v-memo` 指令有效

[错误示例](#vue-valid-v-memo-bad) · [正确示例](#vue-valid-v-memo-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-memo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-memo-bad"></span>

**错误示例**

不带值的 `v-memo` 没有为 Vue 提供依赖表达式，无法判断何时复用子树。

```vue annotate="remove:2"
<template>
  <div v-memo></div>
</template>
```

<span id="vue-valid-v-memo-good"></span>

**正确示例**

`v-memo="[valueA, valueB]"` 提供用于记忆化的依赖数组。

```vue annotate="add:2"
<template>
  <div v-memo="[valueA, valueB]">{{ label }}</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_memo.rs#L27) · [全部规则](all.md)

### `vue/valid-v-model`

要求 `v-model` 指令有效

[错误示例](#vue-valid-v-model-bad) · [正确示例](#vue-valid-v-model-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-model": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-model-bad"></span>

**错误示例**

原生 `<div>` 不能像表单控件一样使用 `v-model`，输入框上不带值的指令也没有可写的目标表达式。

```vue annotate="remove:2,3"
<template>
  <div v-model="value"></div>
  <input v-model />
</template>
```

<span id="vue-valid-v-model-good"></span>

**正确示例**

将 input、select、textarea 或自定义组件绑定到示例中的可写变量。

```vue annotate="add:2,3,4,5"
<template>
  <input v-model="value" />
  <select v-model="selected"></select>
  <textarea v-model="text"></textarea>
  <MyInput v-model="value" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_model.rs#L36) · [全部规则](all.md)

### `vue/valid-v-on`

要求 `v-on` 指令有效

[错误示例](#vue-valid-v-on-bad) · [正确示例](#vue-valid-v-on-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-on": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-on-bad"></span>

**错误示例**

监听器形式缺少事件参数，或缺少所需的处理器或对象表达式。

```vue annotate="remove:2,3,4"
<template>
  <div v-on></div>
  <div @></div>
  <div @click></div>
</template>
```

<span id="vue-valid-v-on-good"></span>

**正确示例**

提供事件及其处理器，或将监听器对象传给无参数的 `v-on`。

```vue annotate="add:2,3"
<template>
  <div @click="handleClick"></div>
  <div v-on="{ click: handleClick }"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_on.rs#L30) · [全部规则](all.md)

### `vue/valid-v-once`

要求 `v-once` 指令有效

[错误示例](#vue-valid-v-once-bad) · [正确示例](#vue-valid-v-once-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-once": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-once-bad"></span>

**错误示例**

`v-once` 设置了值、参数或修饰符，但此指令是无值的单次渲染标记。

```vue annotate="remove:2,3,4"
<template>
<div v-once="foo"></div>
<div v-once:arg></div>
<div v-once.mod></div>
</template>
```

<span id="vue-valid-v-once-good"></span>

**正确示例**

不带值的 `v-once` 将子树标记为只渲染一次，不使用不受支持的语法。

```vue annotate="add:2"
<template>
<div v-once></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_once.rs#L27) · [全部规则](all.md)

### `vue/valid-v-show`

要求 `v-show` 指令有效

[错误示例](#vue-valid-v-show-bad) · [正确示例](#vue-valid-v-show-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-show": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-show-bad"></span>

**错误示例**

`v-show` 缺少可见性表达式，或被放在 `<template>` 上；该元素没有可改变 display 的 DOM 元素。

```vue annotate="remove:2,3"
<template>
  <div v-show></div>
  <template v-show="ready"><div></div></template>
</template>
```

<span id="vue-valid-v-show-good"></span>

**正确示例**

将可见性表达式应用于 `<div>` 等实际渲染的元素。

```vue annotate="add:2,3"
<template>
  <div v-show="ready"></div>
  <div v-show="count > 0"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_show.rs#L28) · [全部规则](all.md)

### `vue/valid-v-slot`

要求 `v-slot` 指令有效

[错误示例](#vue-valid-v-slot-bad) · [正确示例](#vue-valid-v-slot-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-slot": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-slot-bad"></span>

**错误示例**

插槽指令位于原生 `<div>` 上，或与其他默认或具名插槽声明冲突。

```vue annotate="remove:2,3,4"
<template>
  <div v-slot:header></div>
  <MyComponent v-slot v-slot:header />
  <template v-slot:header v-slot:footer />
</template>
```

<span id="vue-valid-v-slot-good"></span>

**正确示例**

在组件自身上声明其默认插槽，或在子 `<template #header>` 上声明其具名插槽。

```vue annotate="add:2,3,4,5"
<template>
  <MyComponent v-slot="{ item }">{{ item }}</MyComponent>
  <MyComponent>
    <template #header>Header</template>
  </MyComponent>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_slot.rs#L29) · [全部规则](all.md)

### `vue/valid-v-text`

要求 `v-text` 指令有效

[错误示例](#vue-valid-v-text-bad) · [正确示例](#vue-valid-v-text-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/valid-v-text": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-valid-v-text-bad"></span>

**错误示例**

`v-text` 缺少文本表达式，或使用了不受支持的参数或修饰符。

```vue annotate="remove:2,3,4"
<template>
<div v-text></div>
<div v-text:arg="foo"></div>
<div v-text.mod="foo"></div>
</template>
```

<span id="vue-valid-v-text-good"></span>

**正确示例**

`v-text="msg"` 在语法上有效。独立的 `vue/no-v-text` 风格规则仍可能要求优先使用插值。

```vue annotate="add:2"
<template>
<div v-text="msg"></div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vue/valid_v_text.rs#L27) · [全部规则](all.md)

### `vue/warn-custom-block`

警告 SFC 文件中的自定义块

[错误示例](#vue-warn-custom-block-bad) · [正确示例](#vue-warn-custom-block-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-block": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-block-bad"></span>

**错误示例**

SFC 包含 `<i18n>` 自定义块，需要普通 template、script 和 style 处理之外的外部集成。

```vue annotate="remove:1,2,3,4"
<i18n>
{ "en": { "hello": "Hello" } }
</i18n>

<template>
  <p>{{ hello }}</p>
</template>
```

<span id="vue-warn-custom-block-good"></span>

**正确示例**

示例使用标准的 template 和 script-setup 块。此可选的可移植性警告并不意味着所有自定义块都是无效 Vue。

```vue annotate="add:4,5,6,7"
<template>
  <p>{{ hello }}</p>
</template>

<script setup lang="ts">
const hello = "Hello";
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_block.rs#L50) · [全部规则](all.md)

### `vue/warn-custom-directive`

警告需要注册的自定义指令

[错误示例](#vue-warn-custom-directive-bad) · [正确示例](#vue-warn-custom-directive-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vue/warn-custom-directive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vue-warn-custom-directive-bad"></span>

**错误示例**

`v-focus`、`v-mask` 和 `v-click-outside` 需要项目特定的指令实现，因此被此可选规范标记。

```vue annotate="remove:2,3,4"
<template>
  <input v-focus />
  <input v-mask="'###-####'" />
  <div v-click-outside="handleClose"></div>
</template>
```

<span id="vue-warn-custom-directive-good"></span>

**正确示例**

示例使用内置的 `v-if`、`v-model` 和 `v-on`。禁用此策略时，正确注册的自定义指令仍可作为有效的 Vue 使用。

```vue annotate="add:2,3,4"
<template>
  <div v-if="ready"></div>
  <input v-model="value" />
  <button type="button" @click="onClick">Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vue/warn_custom_directive.rs#L44) · [全部规则](all.md)
