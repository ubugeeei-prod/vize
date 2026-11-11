---
title: "全部 lint 规则"
description: "Vize linter 提供的全部规则，按类别分组。打开规则页面即可查看它报告的问题、错误与正确示例以及启用方式。 “预设”列给出启用该规则的最小预设，规则页面列出全部预设。各预设包含的内容见[预设](../rules/index.md#预设)。"
---

# 全部 lint 规则

## Vue 规则 (168)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="max-lines"></span>[`max-lines`](./reference/max-lines.md) | 限制文件的行数。 | — |  |
| <span id="vue-a11y-img-alt"></span>[`vue/a11y-img-alt`](./reference/vue-a11y-img-alt.md) | 要求图片提供 alt 属性，以支持无障碍访问。 | — |  |
| <span id="vue-attribute-hyphenation"></span>[`vue/attribute-hyphenation`](./reference/vue-attribute-hyphenation.md) | 规范自定义组件的属性命名风格。 | `happy-path` | ✓ |
| <span id="vue-attribute-order"></span>[`vue/attribute-order`](./reference/vue-attribute-order.md) | 要求属性保持一致的排列顺序。 | `happy-path` |  |
| <span id="vue-block-lang"></span>[`vue/block-lang`](./reference/vue-block-lang.md) | 将 SFC 块的 lang 限制为配置的语言。 | — |  |
| <span id="vue-camelcase"></span>[`vue/camelcase`](./reference/vue-camelcase.md) | 要求模板表达式中的标识符使用 camelCase。 | — |  |
| <span id="vue-comment-directive"></span>[`vue/comment-directive`](./reference/vue-comment-directive.md) | 支持 `<template>` 中的诊断控制注释。 | `essential` |  |
| <span id="vue-component-definition-name-casing"></span>[`vue/component-definition-name-casing`](./reference/vue-component-definition-name-casing.md) | 要求组件定义名称使用 PascalCase 或 kebab-case。 | `happy-path` |  |
| <span id="vue-component-name-in-template-casing"></span>[`vue/component-name-in-template-casing`](./reference/vue-component-name-in-template-casing.md) | 要求模板中的组件名称使用指定的大小写风格。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-dot-notation"></span>[`vue/dot-notation`](./reference/vue-dot-notation.md) | 对静态属性访问优先使用点号。 | — |  |
| <span id="vue-enforce-style-attribute"></span>[`vue/enforce-style-attribute`](./reference/vue-enforce-style-attribute.md) | 将 style 块限制为允许的属性模式。 | — |  |
| <span id="vue-eqeqeq"></span>[`vue/eqeqeq`](./reference/vue-eqeqeq.md) | 要求模板表达式使用严格相等运算符。 | — |  |
| <span id="vue-html-button-has-type"></span>[`vue/html-button-has-type`](./reference/vue-html-button-has-type.md) | 要求 button 元素显式指定有效的 type。 | `nuxt`, `opinionated` |  |
| <span id="vue-html-end-tags"></span>[`vue/html-end-tags`](./reference/vue-html-end-tags.md) | 要求显式书写 HTML 中可以省略的结束标签。 | `happy-path` | ✓ |
| <span id="vue-html-quotes"></span>[`vue/html-quotes`](./reference/vue-html-quotes.md) | 规范 HTML 属性的引号风格。 | `happy-path` | ✓ |
| <span id="vue-html-self-closing"></span>[`vue/html-self-closing`](./reference/vue-html-self-closing.md) | 规范自闭合标签风格。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-jsx-uses-vars"></span>[`vue/jsx-uses-vars`](./reference/vue-jsx-uses-vars.md) | 将 JSX 元素名称中引用的变量视为已使用。 | `essential` |  |
| <span id="vue-match-component-file-name"></span>[`vue/match-component-file-name`](./reference/vue-match-component-file-name.md) | 要求声明的组件名称与文件名匹配。 | — |  |
| <span id="vue-max-lines-per-block"></span>[`vue/max-lines-per-block`](./reference/vue-max-lines-per-block.md) | 限制每个 SFC 块的行数。 | — |  |
| <span id="vue-max-props"></span>[`vue/max-props`](./reference/vue-max-props.md) | 限制 Vue 组件的 props 数量。 | — |  |
| <span id="vue-max-template-complexity"></span>[`vue/max-template-complexity`](./reference/vue-max-template-complexity.md) | 限制组件自身模板的复杂度（圈复杂度和认知复杂度）。 | — |  |
| <span id="vue-max-template-depth"></span>[`vue/max-template-depth`](./reference/vue-max-template-depth.md) | 限制模板的嵌套深度。 | — |  |
| <span id="vue-multi-word-component-names"></span>[`vue/multi-word-component-names`](./reference/vue-multi-word-component-names.md) | 要求组件名称由多个单词组成。 | `essential` |  |
| <span id="vue-mustache-interpolation-spacing"></span>[`vue/mustache-interpolation-spacing`](./reference/vue-mustache-interpolation-spacing.md) | 要求双花括号插值内部的空格保持一致。 | `happy-path` | ✓ |
| <span id="vue-next-tick-style"></span>[`vue/next-tick-style`](./reference/vue-next-tick-style.md) | 要求以 Promise 方式使用 nextTick。 | — |  |
| <span id="vue-no-array-index-key"></span>[`vue/no-array-index-key`](./reference/vue-no-array-index-key.md) | 禁止直接将 v-for 的索引变量用作 :key。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-bare-strings-in-template"></span>[`vue/no-bare-strings-in-template`](./reference/vue-no-bare-strings-in-template.md) | 禁止在模板中直接写入应当国际化的用户可读文本。 | — |  |
| <span id="vue-no-boolean-attr-value"></span>[`vue/no-boolean-attr-value`](./reference/vue-no-boolean-attr-value.md) | 禁止为 HTML 布尔属性显式指定值。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-no-child-content"></span>[`vue/no-child-content`](./reference/vue-no-child-content.md) | 禁止在使用 v-html 或 v-text 时提供子内容。 | `essential` |  |
| <span id="vue-no-console"></span>[`vue/no-console`](./reference/vue-no-console.md) | 禁止模板表达式访问 console。 | — |  |
| <span id="vue-no-constant-condition"></span>[`vue/no-constant-condition`](./reference/vue-no-constant-condition.md) | 禁止模板表达式中的常量条件。 | — |  |
| <span id="vue-no-content-outside-block"></span>[`vue/no-content-outside-block`](./reference/vue-no-content-outside-block.md) | 禁止 SFC 顶层块之外的内容。 | — |  |
| <span id="vue-no-custom-modifiers-on-v-model"></span>[`vue/no-custom-modifiers-on-v-model`](./reference/vue-no-custom-modifiers-on-v-model.md) | 为兼容 Vue 2，禁止在组件的 `v-model` 上使用自定义修饰符。 | — |  |
| <span id="vue-no-define-model-default"></span>[`vue/no-define-model-default`](./reference/vue-no-define-model-default.md) | 禁止 defineModel 中的默认值。 | — |  |
| <span id="vue-no-deprecated-filter"></span>[`vue/no-deprecated-filter`](./reference/vue-no-deprecated-filter.md) | 禁止使用已弃用的 Vue 2 管道运算符过滤器语法。 | `essential` |  |
| <span id="vue-no-deprecated-functional-template"></span>[`vue/no-deprecated-functional-template`](./reference/vue-no-deprecated-functional-template.md) | 禁止在 SFC 的 `<template>` 上使用 `functional` 属性。 | `essential` |  |
| <span id="vue-no-deprecated-html-element-is"></span>[`vue/no-deprecated-html-element-is`](./reference/vue-no-deprecated-html-element-is.md) | 禁止在原生 HTML 元素上使用已弃用的 `is` 属性。 | `essential` |  |
| <span id="vue-no-deprecated-inline-template"></span>[`vue/no-deprecated-inline-template`](./reference/vue-no-deprecated-inline-template.md) | 禁止使用已弃用的 `inline-template` 属性。 | `essential` |  |
| <span id="vue-no-deprecated-router-link-tag-prop"></span>[`vue/no-deprecated-router-link-tag-prop`](./reference/vue-no-deprecated-router-link-tag-prop.md) | 禁止在 &lt;router-link&gt; 上使用 `tag` prop。 | `essential` |  |
| <span id="vue-no-deprecated-scope-attribute"></span>[`vue/no-deprecated-scope-attribute`](./reference/vue-no-deprecated-scope-attribute.md) | 禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性。 | `essential` |  |
| <span id="vue-no-deprecated-slot-attribute"></span>[`vue/no-deprecated-slot-attribute`](./reference/vue-no-deprecated-slot-attribute.md) | 禁止使用已弃用的 `slot` 属性。 | `essential` |  |
| <span id="vue-no-deprecated-slot-scope-attribute"></span>[`vue/no-deprecated-slot-scope-attribute`](./reference/vue-no-deprecated-slot-scope-attribute.md) | 禁止使用已弃用的 `slot-scope` 属性。 | `essential` |  |
| <span id="vue-no-deprecated-v-bind-sync"></span>[`vue/no-deprecated-v-bind-sync`](./reference/vue-no-deprecated-v-bind-sync.md) | 禁止在 `v-bind` 上使用已弃用的 `.sync` 修饰符。 | `essential` |  |
| <span id="vue-no-deprecated-v-is"></span>[`vue/no-deprecated-v-is`](./reference/vue-no-deprecated-v-is.md) | 禁止使用已废弃的 `v-is` 指令。 | `essential` |  |
| <span id="vue-no-deprecated-v-on-native-modifier"></span>[`vue/no-deprecated-v-on-native-modifier`](./reference/vue-no-deprecated-v-on-native-modifier.md) | 禁止在 `v-on` 上使用已弃用的 `.native` 修饰符。 | `essential` |  |
| <span id="vue-no-deprecated-v-on-number-modifiers"></span>[`vue/no-deprecated-v-on-number-modifiers`](./reference/vue-no-deprecated-v-on-number-modifiers.md) | 禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符。 | `essential` | ✓ |
| <span id="vue-no-dupe-v-else-if"></span>[`vue/no-dupe-v-else-if`](./reference/vue-no-dupe-v-else-if.md) | 禁止在 `v-if` / `v-else-if` 链中使用重复条件。 | `essential` |  |
| <span id="vue-no-duplicate-attributes"></span>[`vue/no-duplicate-attributes`](./reference/vue-no-duplicate-attributes.md) | 禁止在同一元素上重复声明属性。 | `essential` |  |
| <span id="vue-no-empty-component-block"></span>[`vue/no-empty-component-block`](./reference/vue-no-empty-component-block.md) | 禁止空的 SFC 块。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-empty-pattern"></span>[`vue/no-empty-pattern`](./reference/vue-no-empty-pattern.md) | 禁止模板表达式中的空解构模式。 | — |  |
| <span id="vue-no-implicit-coercion"></span>[`vue/no-implicit-coercion`](./reference/vue-no-implicit-coercion.md) | 禁止模板表达式中的隐式类型转换。 | — |  |
| <span id="vue-no-inline-style"></span>[`vue/no-inline-style`](./reference/vue-no-inline-style.md) | 不建议使用内联 style 属性。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-invalid-html-attribute"></span>[`vue/no-invalid-html-attribute`](./reference/vue-no-invalid-html-attribute.md) | 禁止 HTML 属性使用无效的静态值。 | `happy-path` |  |
| <span id="vue-no-irregular-whitespace"></span>[`vue/no-irregular-whitespace`](./reference/vue-no-irregular-whitespace.md) | 检测模板表达式中的异常空白字符。 | — |  |
| <span id="vue-no-literals-in-template"></span>[`vue/no-literals-in-template`](./reference/vue-no-literals-in-template.md) | 禁止 v-bind 中的对象、数组和函数字面量。 | — |  |
| <span id="vue-no-lone-template"></span>[`vue/no-lone-template`](./reference/vue-no-lone-template.md) | 禁止不必要的 `<template>` 元素。 | `happy-path` |  |
| <span id="vue-no-loss-of-precision"></span>[`vue/no-loss-of-precision`](./reference/vue-no-loss-of-precision.md) | 禁止会丢失精度的数字字面量。 | — |  |
| <span id="vue-no-multi-spaces"></span>[`vue/no-multi-spaces`](./reference/vue-no-multi-spaces.md) | 禁止连续多个空格。 | `happy-path` | ✓ |
| <span id="vue-no-multiple-objects-in-class"></span>[`vue/no-multiple-objects-in-class`](./reference/vue-no-multiple-objects-in-class.md) | 禁止在 :class 数组绑定中使用多个对象字面量。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-multiple-template-root"></span>[`vue/no-multiple-template-root`](./reference/vue-no-multiple-template-root.md) | 禁止模板包含多个根节点。 | — |  |
| <span id="vue-no-mutating-props"></span>[`vue/no-mutating-props`](./reference/vue-no-mutating-props.md) | 禁止修改组件 props。 | `essential` |  |
| <span id="vue-no-negated-condition"></span>[`vue/no-negated-condition`](./reference/vue-no-negated-condition.md) | 禁止以否定条件开始三元表达式。 | — |  |
| <span id="vue-no-negated-v-if-condition"></span>[`vue/no-negated-v-if-condition`](./reference/vue-no-negated-v-if-condition.md) | 禁止在含有 v-else 的条件链中使用否定的 v-if 条件。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-non-component-keep-alive-child"></span>[`vue/no-non-component-keep-alive-child`](./reference/vue-no-non-component-keep-alive-child.md) | 禁止在 `<KeepAlive>` 的直属子层使用普通元素包装。 | `essential` |  |
| <span id="vue-no-preprocessor-lang"></span>[`vue/no-preprocessor-lang`](./reference/vue-no-preprocessor-lang.md) | 不建议使用 CSS 预处理器，优先使用现代 CSS。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-props-shadow"></span>[`vue/no-props-shadow`](./reference/vue-no-props-shadow.md) | 禁止 script setup 声明遮蔽 prop。 | — |  |
| <span id="vue-no-ref-object-reactivity-loss"></span>[`vue/no-ref-object-reactivity-loss`](./reference/vue-no-ref-object-reactivity-loss.md) | 禁止在同一作用域中复制 ref 的值而丢失后续响应式更新。 | — |  |
| <span id="vue-no-reserved-component-names"></span>[`vue/no-reserved-component-names`](./reference/vue-no-reserved-component-names.md) | 禁止使用保留名称作为组件名称。 | `essential` |  |
| <span id="vue-no-restricted-block"></span>[`vue/no-restricted-block`](./reference/vue-no-restricted-block.md) | 禁止使用配置的块。 | — |  |
| <span id="vue-no-restricted-call-after-await"></span>[`vue/no-restricted-call-after-await`](./reference/vue-no-restricted-call-after-await.md) | 禁止在 await 之后调用受限制的方法。 | — |  |
| <span id="vue-no-restricted-class"></span>[`vue/no-restricted-class`](./reference/vue-no-restricted-class.md) | 禁止使用配置的 class。 | — |  |
| <span id="vue-no-restricted-component-names"></span>[`vue/no-restricted-component-names`](./reference/vue-no-restricted-component-names.md) | 禁止使用配置的组件名。 | — |  |
| <span id="vue-no-restricted-component-options"></span>[`vue/no-restricted-component-options`](./reference/vue-no-restricted-component-options.md) | 禁止使用配置的组件选项。 | — |  |
| <span id="vue-no-restricted-custom-event"></span>[`vue/no-restricted-custom-event`](./reference/vue-no-restricted-custom-event.md) | 禁止使用配置的自定义事件。 | — |  |
| <span id="vue-no-restricted-html-elements"></span>[`vue/no-restricted-html-elements`](./reference/vue-no-restricted-html-elements.md) | 禁止使用配置的元素。 | — |  |
| <span id="vue-no-restricted-props"></span>[`vue/no-restricted-props`](./reference/vue-no-restricted-props.md) | 禁止使用配置的 props。 | — |  |
| <span id="vue-no-restricted-static-attribute"></span>[`vue/no-restricted-static-attribute`](./reference/vue-no-restricted-static-attribute.md) | 禁止使用配置的静态属性。 | — |  |
| <span id="vue-no-restricted-syntax"></span>[`vue/no-restricted-syntax`](./reference/vue-no-restricted-syntax.md) | 禁止模板表达式中配置的语法。 | — |  |
| <span id="vue-no-restricted-v-bind"></span>[`vue/no-restricted-v-bind`](./reference/vue-no-restricted-v-bind.md) | 禁止受限的 v-bind 参数。 | — |  |
| <span id="vue-no-restricted-v-on"></span>[`vue/no-restricted-v-on`](./reference/vue-no-restricted-v-on.md) | 禁止使用配置的 v-on 参数。 | — |  |
| <span id="vue-no-root-v-if"></span>[`vue/no-root-v-if`](./reference/vue-no-root-v-if.md) | 禁止在模板的唯一根元素上使用 v-if。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-script-non-standard-lang"></span>[`vue/no-script-non-standard-lang`](./reference/vue-no-script-non-standard-lang.md) | 不建议使用非标准的 script lang 值。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-setup-props-reactivity-loss"></span>[`vue/no-setup-props-reactivity-loss`](./reference/vue-no-setup-props-reactivity-loss.md) | 禁止通过复制 setup 的 prop 值而丢失后续响应式更新。 | — |  |
| <span id="vue-no-shadow-native-events"></span>[`vue/no-shadow-native-events`](./reference/vue-no-shadow-native-events.md) | 禁止组件事件名与原生事件冲突。 | — |  |
| <span id="vue-no-sparse-arrays"></span>[`vue/no-sparse-arrays`](./reference/vue-no-sparse-arrays.md) | 禁止模板表达式中的稀疏数组。 | — |  |
| <span id="vue-no-src-attribute"></span>[`vue/no-src-attribute`](./reference/vue-no-src-attribute.md) | 不建议在 SFC 块上使用 src 属性。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-static-inline-styles"></span>[`vue/no-static-inline-styles`](./reference/vue-no-static-inline-styles.md) | 禁止静态内联 style 属性。 | — |  |
| <span id="vue-no-template-key"></span>[`vue/no-template-key`](./reference/vue-no-template-key.md) | 禁止在 `<template>` 上使用 `key` 属性。 | `essential` |  |
| <span id="vue-no-template-lang"></span>[`vue/no-template-lang`](./reference/vue-no-template-lang.md) | 不建议在 template 块上使用 lang 属性。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-template-shadow"></span>[`vue/no-template-shadow`](./reference/vue-no-template-shadow.md) | 禁止使用遮蔽外层作用域变量的变量名称。 | `happy-path` |  |
| <span id="vue-no-template-target-blank"></span>[`vue/no-template-target-blank`](./reference/vue-no-template-target-blank.md) | 禁止使用 target="_blank" 而不设置 rel="noopener noreferrer"。 | `happy-path` |  |
| <span id="vue-no-textarea-mustache"></span>[`vue/no-textarea-mustache`](./reference/vue-no-textarea-mustache.md) | 禁止在 `<textarea>` 内使用双花括号插值。 | `essential` |  |
| <span id="vue-no-this-in-before-route-enter"></span>[`vue/no-this-in-before-route-enter`](./reference/vue-no-this-in-before-route-enter.md) | 禁止在 `beforeRouteEnter` 中通过 `this` 访问尚未创建的组件实例。 | `happy-path` |  |
| <span id="vue-no-undefined-refs"></span>[`vue/no-undefined-refs`](./reference/vue-no-undefined-refs.md) | 禁止在模板中引用未定义的变量。 | `happy-path` |  |
| <span id="vue-no-unsafe-url"></span>[`vue/no-unsafe-url`](./reference/vue-no-unsafe-url.md) | 警告可能不安全的 URL 绑定。 | `essential` |  |
| <span id="vue-no-unsandboxed-iframe"></span>[`vue/no-unsandboxed-iframe`](./reference/vue-no-unsandboxed-iframe.md) | 要求 iframe 元素具有 sandbox 属性。 | `happy-path` |  |
| <span id="vue-no-unsupported-features"></span>[`vue/no-unsupported-features`](./reference/vue-no-unsupported-features.md) | 禁止在指定的 Vue.js 版本上使用尚未支持的语法。 | — |  |
| <span id="vue-no-unused-components"></span>[`vue/no-unused-components`](./reference/vue-no-unused-components.md) | 禁止注册未在模板中使用的组件。 | `essential` |  |
| <span id="vue-no-unused-properties"></span>[`vue/no-unused-properties`](./reference/vue-no-unused-properties.md) | 禁止 defineProps 中定义的未使用属性。 | `happy-path` |  |
| <span id="vue-no-unused-refs"></span>[`vue/no-unused-refs`](./reference/vue-no-unused-refs.md) | 报告从未在 &lt;script&gt; 中引用的模板 ref（ref="x"）。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-script-vars"></span>[`vue/no-unused-script-vars`](./reference/vue-no-unused-script-vars.md) | 禁止未使用的脚本变量、导入和参数。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-unused-setup-bindings"></span>[`vue/no-unused-setup-bindings`](./reference/vue-no-unused-setup-bindings.md) | 禁止从未读取的 script setup 绑定。 | `happy-path` |  |
| <span id="vue-no-unused-vars"></span>[`vue/no-unused-vars`](./reference/vue-no-unused-vars.md) | 禁止 v-for 和 v-slot 指令中未使用的变量定义。 | `essential` |  |
| <span id="vue-no-use-v-else-with-v-for"></span>[`vue/no-use-v-else-with-v-for`](./reference/vue-no-use-v-else-with-v-for.md) | 禁止在同一元素上使用 `v-else-if` 或 `v-else` 与 `v-for`。 | `happy-path` |  |
| <span id="vue-no-use-v-if-with-v-for"></span>[`vue/no-use-v-if-with-v-for`](./reference/vue-no-use-v-if-with-v-for.md) | 禁止在同一元素上使用 `v-if` 与 `v-for`。 | `essential` |  |
| <span id="vue-no-useless-concat"></span>[`vue/no-useless-concat`](./reference/vue-no-useless-concat.md) | 禁止字符串字面量之间无意义的拼接。 | — |  |
| <span id="vue-no-useless-mustaches"></span>[`vue/no-useless-mustaches`](./reference/vue-no-useless-mustaches.md) | 禁止表达式为常量字符串字面量的双花括号插值。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-useless-template-attributes"></span>[`vue/no-useless-template-attributes`](./reference/vue-no-useless-template-attributes.md) | 禁止在 `<template>` 元素上使用不起作用的属性。 | `essential` |  |
| <span id="vue-no-useless-v-bind"></span>[`vue/no-useless-v-bind`](./reference/vue-no-useless-v-bind.md) | 禁止值为普通字符串字面量的 v-bind。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-for-template-key"></span>[`vue/no-v-for-template-key`](./reference/vue-no-v-for-template-key.md) | 为兼容 Vue 2，要求将 `template v-for` 的 key 放在实际子元素上。 | — |  |
| <span id="vue-no-v-for-template-key-on-child"></span>[`vue/no-v-for-template-key-on-child`](./reference/vue-no-v-for-template-key-on-child.md) | 禁止在 `<template v-for>` 的子节点上使用 `key`。 | `essential` |  |
| <span id="vue-no-v-html"></span>[`vue/no-v-html`](./reference/vue-no-v-html.md) | 警告使用 v-html，以避免 XSS 漏洞。 | `essential` |  |
| <span id="vue-no-v-text"></span>[`vue/no-v-text`](./reference/vue-no-v-text.md) | 禁止 v-text 指令，优先使用双花括号插值。 | `nuxt`, `opinionated` |  |
| <span id="vue-no-v-text-v-html-on-component"></span>[`vue/no-v-text-v-html-on-component`](./reference/vue-no-v-text-v-html-on-component.md) | 禁止在组件元素上使用 v-text / v-html。 | `essential` |  |
| <span id="vue-object-shorthand"></span>[`vue/object-shorthand`](./reference/vue-object-shorthand.md) | 要求模板对象使用属性简写。 | — |  |
| <span id="vue-permitted-contents"></span>[`vue/permitted-contents`](./reference/vue-permitted-contents.md) | 要求符合 HTML 内容模型规则。 | `happy-path` |  |
| <span id="vue-prefer-const"></span>[`vue/prefer-const`](./reference/vue-prefer-const.md) | 要求脚本和模板都不会重新赋值的绑定使用 const。 | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-prop-type-boolean-first"></span>[`vue/prefer-prop-type-boolean-first`](./reference/vue-prefer-prop-type-boolean-first.md) | 要求在组件 prop 的类型构造函数数组中将 `Boolean` 放在首位。 | — |  |
| <span id="vue-prefer-props-shorthand"></span>[`vue/prefer-props-shorthand`](./reference/vue-prefer-props-shorthand.md) | 建议使用 props 简写语法（Vue 3.4+）。 | `nuxt`, `opinionated` | ✓ |
| <span id="vue-prefer-template"></span>[`vue/prefer-template`](./reference/vue-prefer-template.md) | 优先使用模板字符串而非字符串拼接。 | — |  |
| <span id="vue-prefer-true-attribute-shorthand"></span>[`vue/prefer-true-attribute-shorthand`](./reference/vue-prefer-true-attribute-shorthand.md) | 优先使用绑定到 `true` 的布尔属性简写。 | `nuxt`, `opinionated` |  |
| <span id="vue-prefer-v-model"></span>[`vue/prefer-v-model`](./reference/vue-prefer-v-model.md) | 用 v-model 代替匹配的 bind 与 update 监听器。 | — |  |
| <span id="vue-prop-name-casing"></span>[`vue/prop-name-casing`](./reference/vue-prop-name-casing.md) | 规范所声明 prop 名称的大小写风格。 | `happy-path` |  |
| <span id="vue-quote-props"></span>[`vue/quote-props`](./reference/vue-quote-props.md) | 禁止不必要的带引号对象属性。 | — |  |
| <span id="vue-require-component-is"></span>[`vue/require-component-is`](./reference/vue-require-component-is.md) | 要求 `<component>` 元素具有 `v-bind:is`。 | `essential` |  |
| <span id="vue-require-component-registration"></span>[`vue/require-component-registration`](./reference/vue-require-component-registration.md) | 要求显式导入或注册组件。 | `opinionated` |  |
| <span id="vue-require-default-export"></span>[`vue/require-default-export`](./reference/vue-require-default-export.md) | 要求组件的 script 作为默认导出。 | — |  |
| <span id="vue-require-expose"></span>[`vue/require-expose`](./reference/vue-require-expose.md) | 要求 Options API 组件通过 `expose` 显式声明公开属性。 | — |  |
| <span id="vue-require-macro-variable-name"></span>[`vue/require-macro-variable-name`](./reference/vue-require-macro-variable-name.md) | 要求编译器宏使用约定的变量名。 | — |  |
| <span id="vue-require-prop-comment"></span>[`vue/require-prop-comment`](./reference/vue-require-prop-comment.md) | 要求每个声明的 prop 都有 JSDoc 注释。 | — |  |
| <span id="vue-require-scoped-style"></span>[`vue/require-scoped-style`](./reference/vue-require-scoped-style.md) | 要求 style 标签具有 scoped 属性。 | `happy-path` |  |
| <span id="vue-require-toggle-inside-transition"></span>[`vue/require-toggle-inside-transition`](./reference/vue-require-toggle-inside-transition.md) | 要求 `<transition>` 包裹的元素具有切换条件。 | `essential` |  |
| <span id="vue-require-v-for-key"></span>[`vue/require-v-for-key`](./reference/vue-require-v-for-key.md) | 要求 `v-for` 指令搭配 `v-bind:key`。 | `essential` |  |
| <span id="vue-scoped-event-names"></span>[`vue/scoped-event-names`](./reference/vue-scoped-event-names.md) | 建议使用 context:event 格式的作用域事件名称。 | `nuxt`, `opinionated` |  |
| <span id="vue-sfc-element-order"></span>[`vue/sfc-element-order`](./reference/vue-sfc-element-order.md) | 要求 SFC 顶层元素保持一致的顺序。 | `happy-path` |  |
| <span id="vue-single-style-block"></span>[`vue/single-style-block`](./reference/vue-single-style-block.md) | 建议只使用一个 style 块。 | `happy-path` |  |
| <span id="vue-slot-name-casing"></span>[`vue/slot-name-casing`](./reference/vue-slot-name-casing.md) | 要求渲染的插槽出口静态名称采用配置的大小写格式。 | `nuxt`, `opinionated` |  |
| <span id="vue-sort-keys"></span>[`vue/sort-keys`](./reference/vue-sort-keys.md) | 按配置顺序排列组件选项中的键。 | — |  |
| <span id="vue-this-in-template"></span>[`vue/this-in-template`](./reference/vue-this-in-template.md) | 禁止在模板表达式中使用 `this.`。 | `happy-path` |  |
| <span id="vue-use-unique-element-ids"></span>[`vue/use-unique-element-ids`](./reference/vue-use-unique-element-ids.md) | 要求使用 useId() 生成唯一元素 ID，而不是静态字面量。 | `nuxt`, `opinionated` |  |
| <span id="vue-use-v-on-exact"></span>[`vue/use-v-on-exact`](./reference/vue-use-v-on-exact.md) | 存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`。 | `essential` |  |
| <span id="vue-v-bind-style"></span>[`vue/v-bind-style`](./reference/vue-v-bind-style.md) | 规范 `v-bind` 指令风格。 | `happy-path` | ✓ |
| <span id="vue-v-if-else-key"></span>[`vue/v-if-else-key`](./reference/vue-v-if-else-key.md) | 要求重复的条件组件具有 key。 | — |  |
| <span id="vue-v-on-event-hyphenation"></span>[`vue/v-on-event-hyphenation`](./reference/vue-v-on-event-hyphenation.md) | 要求组件上 v-on 中的自定义事件名称使用连字符。 | `happy-path` | ✓ |
| <span id="vue-v-on-handler-style"></span>[`vue/v-on-handler-style`](./reference/vue-v-on-handler-style.md) | 要求 v-on 处理器写为方法引用。 | `nuxt`, `opinionated` |  |
| <span id="vue-v-on-style"></span>[`vue/v-on-style`](./reference/vue-v-on-style.md) | 规范 `v-on` 指令风格。 | `happy-path` | ✓ |
| <span id="vue-v-slot-style"></span>[`vue/v-slot-style`](./reference/vue-v-slot-style.md) | 规范 `v-slot` 指令风格。 | `happy-path` | ✓ |
| <span id="vue-valid-attribute-name"></span>[`vue/valid-attribute-name`](./reference/vue-valid-attribute-name.md) | 要求属性名称有效。 | `essential` |  |
| <span id="vue-valid-model-definition"></span>[`vue/valid-model-definition`](./reference/vue-valid-model-definition.md) | 要求 Vue 2 的 `model` 选项使用有效的键。 | — |  |
| <span id="vue-valid-template-root"></span>[`vue/valid-template-root`](./reference/vue-valid-template-root.md) | 要求 `<template>` 根节点符合 Vue 3 片段语义。 | `essential` |  |
| <span id="vue-valid-v-bind"></span>[`vue/valid-v-bind`](./reference/vue-valid-v-bind.md) | 要求 `v-bind` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-cloak"></span>[`vue/valid-v-cloak`](./reference/vue-valid-v-cloak.md) | 要求 `v-cloak` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-else"></span>[`vue/valid-v-else`](./reference/vue-valid-v-else.md) | 要求 `v-else` 指令有效。 | `essential` | ✓ |
| <span id="vue-valid-v-else-if"></span>[`vue/valid-v-else-if`](./reference/vue-valid-v-else-if.md) | 要求 `v-else-if` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-for"></span>[`vue/valid-v-for`](./reference/vue-valid-v-for.md) | 要求 `v-for` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-html"></span>[`vue/valid-v-html`](./reference/vue-valid-v-html.md) | 要求 `v-html` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-if"></span>[`vue/valid-v-if`](./reference/vue-valid-v-if.md) | 要求 `v-if` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-is"></span>[`vue/valid-v-is`](./reference/vue-valid-v-is.md) | 要求 `v-is` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-memo"></span>[`vue/valid-v-memo`](./reference/vue-valid-v-memo.md) | 要求 `v-memo` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-model"></span>[`vue/valid-v-model`](./reference/vue-valid-v-model.md) | 要求 `v-model` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-on"></span>[`vue/valid-v-on`](./reference/vue-valid-v-on.md) | 要求 `v-on` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-once"></span>[`vue/valid-v-once`](./reference/vue-valid-v-once.md) | 要求 `v-once` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-pre"></span>[`vue/valid-v-pre`](./reference/vue-valid-v-pre.md) | 要求 `v-pre` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-show"></span>[`vue/valid-v-show`](./reference/vue-valid-v-show.md) | 要求 `v-show` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-slot"></span>[`vue/valid-v-slot`](./reference/vue-valid-v-slot.md) | 要求 `v-slot` 指令有效。 | `essential` |  |
| <span id="vue-valid-v-text"></span>[`vue/valid-v-text`](./reference/vue-valid-v-text.md) | 要求 `v-text` 指令有效。 | `essential` |  |
| <span id="vue-warn-custom-block"></span>[`vue/warn-custom-block`](./reference/vue-warn-custom-block.md) | 警告 SFC 文件中的自定义块。 | `nuxt`, `opinionated` |  |
| <span id="vue-warn-custom-directive"></span>[`vue/warn-custom-directive`](./reference/vue-warn-custom-directive.md) | 警告需要注册的自定义指令。 | `nuxt`, `opinionated` |  |

## 类型与脚本规则 (123)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | 要求组件 `name` 选项使用 PascalCase。 | `opinionated`, `nuxt` |  |
| <span id="script-consistent-props-emits-key-quotes"></span>[`script/consistent-props-emits-key-quotes`](./reference/script-consistent-props-emits-key-quotes.md) | 统一 props 和 emits 声明中带引号的键。 | — |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | 要求发出的自定义事件名称使用配置的大小写形式。 | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | 要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式。 | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | 要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序。 | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | 要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式。 | `opinionated`, `nuxt` |  |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | 要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致。 | `happy-path` |  |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | 禁止将箭头函数用作 Options API watch 处理器。 | `essential` |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | 禁止计算属性中的异步函数。 | `essential` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | 禁止为 Boolean prop 设置默认值。 | `opinionated`, `nuxt` |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | 禁止在组件的 data 函数中使用计算属性。 | `essential` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | 禁止在 defineProps 中进行深层嵌套解构。 | `opinionated`, `nuxt` |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | 禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）。 | `essential` |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | 禁止使用 Vue 3 中已移除的 Vue 2 $set 和 $delete。 | `essential` |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | 禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子。 | `essential` | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | 禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）。 | `essential` |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | 禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）。 | `essential` |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | 禁止已移除的 Vue 2 事件 API（$on / $off / $once）。 | `essential` |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | 禁止使用 Vue 3 中已移除的 Vue 2 model 选项。 | `essential` |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | 禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）。 | `essential` |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | 禁止使用 Vue 3 中已移除的 Vue.config.keyCodes。 | `essential` |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | 禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键。 | `essential` |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | 标记重复应用透传属性的组件。 | `essential` |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | 禁止 &lt;script setup&gt; 内的 export 语句。 | `essential` |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | 禁止在 await 之后调用 expose() 和 defineExpose()。 | `essential` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）。 | `opinionated` |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | 禁止导入自动可用的 Vue 编译器宏。 | `essential` |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | 禁止从 Vue 内部模块导入。 | `essential` |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | 禁止在 setup() 中 await 之后注册生命周期钩子。 | `essential` |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | 禁止向作用域插槽函数调用传入多个参数。 | `happy-path` |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | 禁止面向 Vapor 的组件使用 nextTick()。 | — |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | 禁止 Vapor 模式中的 Options API 模式。 | `opinionated` |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | 标记 Options API 组件选项名称中的疑似拼写错误。 | `essential` |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | 禁止导致响应性丢失的响应式对象解构。 | — |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | 要求将 ref 绑定变量用作操作数时通过 `.value` 访问。 | `essential` |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | 禁止 prop 同时具有 required: true 和默认值。 | `essential` |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | 禁止使用 Vue 编译器保留标识符。 | `essential` |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | 禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键。 | `essential` |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | 禁止在组件 props 声明中使用保留名称。 | `essential` |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | 禁止直接引用必须通过类型化包装器访问的运行时环境全局变量。 | — |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | 禁止项目配置的 object.property 成员访问。 | — |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | 禁止 Options API 计算 getter 中的副作用。 | `essential` |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | 禁止顶层 ref/reactive，以防止跨请求状态污染。 | — |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | 禁止在 setup 或 render 函数内定义组件。 | `happy-path` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | 标记已声明却从未发出的事件。 | `opinionated`, `nuxt` |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | 禁止像方法一样调用 Options API 计算属性。 | `essential` |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | 禁止在 setup() 中 await 之后注册未保留停止句柄的侦听器。 | `essential` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | 不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）。 | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | 禁止在一个文件中定义多个组件。 | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | 强制组件选项的属性遵循默认顺序。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | 响应式派生状态优先使用 computed()。 | — |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | 优先使用从 defineEmits 捕获的函数而不是模板 $emit。 | — | ✓ |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | 优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | 优先从 'vue' 导入，而不是内部包。 | `essential` | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | 建议使用 ref() 而不是 reactive() 管理状态。 | — |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | 建议使用 useAttrs() 而不是 context.attrs。 | — |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | 建议使用 useId() 生成唯一 ID（Vue 3.5+）。 | — |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | 建议使用 useSlots() 而不是 context.slots。 | — |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | 模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+）。 | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | 要求每个可选的非 Boolean prop 具有默认值。 | `opinionated`, `nuxt` |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | 要求 TypeScript SFC 的默认导出使用 defineComponent()。 | `happy-path` |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | 要求发出的事件在 defineEmits 或 emits 选项中声明。 | `happy-path` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | 要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型。 | `opinionated`, `nuxt` |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | 要求函数具有返回类型注解。 | — |  |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | 要求 prop 的 `type` 值是构造器，而不是字符串字面量。 | `essential` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | 要求每个 prop 声明类型。 | `opinionated` |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | 要求组件的 render 函数返回一个值。 | `essential` |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | 要求将 $slots 的属性作为函数调用。 | `essential` |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | 建议使用 Symbol 作为 provide/inject 的注入键。 | — |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | 要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型。 | `opinionated`, `nuxt` |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | 要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数。 | — |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | 要求 prop 默认值符合声明类型。 | `essential` |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | 要求每个计算 getter 返回值。 | `essential` |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | 要求每个 Options API emits 校验函数返回值。 | `essential` |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | 要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）。 | `essential` |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | 要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots）。 | `essential` |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | 要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）。 | `essential` |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | 要求 nextTick() 调用结果被等待、链式处理，或提供回调。 | `essential` |  |
| <span id="type-await-thenable"></span>[`type/await-thenable`](./reference/type-await-thenable.md) | 禁止 await 非 thenable 的值。 | `nuxt`, `opinionated` |  |
| <span id="type-no-array-delete"></span>[`type/no-array-delete`](./reference/type-no-array-delete.md) | 禁止对类数组值使用 delete 删除元素。 | `nuxt`, `opinionated` |  |
| <span id="type-no-base-to-string"></span>[`type/no-base-to-string`](./reference/type-no-base-to-string.md) | 禁止把会回退到 Object.prototype.toString() 的值转换成字符串。 | `nuxt`, `opinionated` |  |
| <span id="type-no-confusing-void-expression"></span>[`type/no-confusing-void-expression`](./reference/type-no-confusing-void-expression.md) | 禁止在容易混淆的值位置使用 void 类型的表达式。 | `nuxt`, `opinionated` |  |
| <span id="type-no-deprecated"></span>[`type/no-deprecated`](./reference/type-no-deprecated.md) | 禁止使用已弃用的声明。 | `nuxt`, `opinionated` |  |
| <span id="type-no-duplicate-type-constituents"></span>[`type/no-duplicate-type-constituents`](./reference/type-no-duplicate-type-constituents.md) | 禁止联合类型或交叉类型中的重复成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | 禁止悬空（未处理）的 Promise。 | `nuxt`, `opinionated` |  |
| <span id="type-no-for-in-array"></span>[`type/no-for-in-array`](./reference/type-no-for-in-array.md) | 禁止对类数组值使用 for-in 遍历。 | `nuxt`, `opinionated` |  |
| <span id="type-no-generated-empty-object-type"></span>[`type/no-generated-empty-object-type`](./reference/type-no-generated-empty-object-type.md) | 禁止由类型运算生成的空对象类型。 | `nuxt`, `opinionated` |  |
| <span id="type-no-implied-eval"></span>[`type/no-implied-eval`](./reference/type-no-implied-eval.md) | 禁止基于字符串的动态代码执行 API。 | `nuxt`, `opinionated` |  |
| <span id="type-no-meaningless-void-operator"></span>[`type/no-meaningless-void-operator`](./reference/type-no-meaningless-void-operator.md) | 禁止对已经是 void 的表达式使用 void 运算符。 | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-promises"></span>[`type/no-misused-promises`](./reference/type-no-misused-promises.md) | 禁止在并非为处理 Promise 而设计的位置使用 Promise。 | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-spread"></span>[`type/no-misused-spread`](./reference/type-no-misused-spread.md) | 禁止把值展开到不兼容的位置。 | `nuxt`, `opinionated` |  |
| <span id="type-no-mixed-enums"></span>[`type/no-mixed-enums`](./reference/type-no-mixed-enums.md) | 禁止在枚举中混用字符串成员和数值成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | 禁止赋值和调用中对响应式值取普通快照。 | `nuxt`, `opinionated` |  |
| <span id="type-no-redundant-type-constituents"></span>[`type/no-redundant-type-constituents`](./reference/type-no-redundant-type-constituents.md) | 禁止联合类型和交叉类型中不起作用或覆盖类型信息的成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-boolean-literal-compare"></span>[`type/no-unnecessary-boolean-literal-compare`](./reference/type-no-unnecessary-boolean-literal-compare.md) | 禁止与布尔字面量进行不必要的比较。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-condition"></span>[`type/no-unnecessary-condition`](./reference/type-no-unnecessary-condition.md) | 禁止始终为真、始终为假、never 或始终为 nullish 的条件。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-template-expression"></span>[`type/no-unnecessary-template-expression`](./reference/type-no-unnecessary-template-expression.md) | 禁止用不必要的模板字符串只包裹一个表达式。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-arguments"></span>[`type/no-unnecessary-type-arguments`](./reference/type-no-unnecessary-type-arguments.md) | 禁止与默认值相同的类型参数。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-assertion"></span>[`type/no-unnecessary-type-assertion`](./reference/type-no-unnecessary-type-assertion.md) | 禁止不改变表达式类型的类型断言。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-conversion"></span>[`type/no-unnecessary-type-conversion`](./reference/type-no-unnecessary-type-conversion.md) | 禁止不改变表达式类型或值的转换。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-argument"></span>[`type/no-unsafe-argument`](./reference/type-no-unsafe-argument.md) | 禁止用 `any` 类型的值调用函数。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-assignment"></span>[`type/no-unsafe-assignment`](./reference/type-no-unsafe-assignment.md) | 禁止把 any 类型的值赋给更具体的目标。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-call"></span>[`type/no-unsafe-call`](./reference/type-no-unsafe-call.md) | 禁止调用类型为 any 的值。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-assignment"></span>[`type/no-unsafe-enum-assignment`](./reference/type-no-unsafe-enum-assignment.md) | 禁止把非枚举值赋给枚举。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-comparison"></span>[`type/no-unsafe-enum-comparison`](./reference/type-no-unsafe-enum-comparison.md) | 禁止把枚举值与另一枚举域的值比较。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-member-access"></span>[`type/no-unsafe-member-access`](./reference/type-no-unsafe-member-access.md) | 禁止访问类型为 any 的值的成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-return"></span>[`type/no-unsafe-return`](./reference/type-no-unsafe-return.md) | 禁止从函数返回 any 类型的值。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | 禁止解析为不安全类型的模板绑定。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-unary-minus"></span>[`type/no-unsafe-unary-minus`](./reference/type-no-unsafe-unary-minus.md) | 禁止对非 number 且非 bigint 的值使用一元负号。 | `nuxt`, `opinionated` |  |
| <span id="type-only-throw-error"></span>[`type/only-throw-error`](./reference/type-only-throw-error.md) | 要求抛出的值是 Error 一类的值。 | `nuxt`, `opinionated` |  |
| <span id="type-prefer-promise-reject-errors"></span>[`type/prefer-promise-reject-errors`](./reference/type-prefer-promise-reject-errors.md) | 要求 Promise 的拒绝原因是 Error 一类的值。 | `nuxt`, `opinionated` |  |
| <span id="type-prefer-reduce-type-parameter"></span>[`type/prefer-reduce-type-parameter`](./reference/type-prefer-reduce-type-parameter.md) | 推荐使用 Array#reduce 的类型参数，而不是对初始值做断言。 | `nuxt`, `opinionated` |  |
| <span id="type-prefer-return-this-type"></span>[`type/prefer-return-this-type`](./reference/type-prefer-return-this-type.md) | 要求在只返回 `this` 类型时使用 `this`。 | `nuxt`, `opinionated` |  |
| <span id="type-related-getter-setter-pairs"></span>[`type/related-getter-setter-pairs`](./reference/type-related-getter-setter-pairs.md) | 要求成对的 getter 和 setter 使用兼容的类型。 | `nuxt`, `opinionated` |  |
| <span id="type-require-await"></span>[`type/require-await`](./reference/type-require-await.md) | 禁止不包含 await 表达式的 async 函数。 | `nuxt`, `opinionated` |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | 要求 defineEmits 具有类型定义。 | `happy-path` |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | 要求 defineProps 具有类型定义。 | `happy-path` |  |
| <span id="type-restrict-plus-operands"></span>[`type/restrict-plus-operands`](./reference/type-restrict-plus-operands.md) | 要求加号的操作数明确兼容。 | `nuxt`, `opinionated` |  |
| <span id="type-restrict-template-expressions"></span>[`type/restrict-template-expressions`](./reference/type-restrict-template-expressions.md) | 要求模板字符串中的表达式与字符串兼容。 | `nuxt`, `opinionated` |  |
| <span id="type-return-await"></span>[`type/return-await`](./reference/type-return-await.md) | 要求根据上下文和配置一致地使用 `return await`。 | `nuxt`, `opinionated` |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | 要求脚本和模板条件使用安全的布尔表达式。 | — |  |
| <span id="type-unbound-method"></span>[`type/unbound-method`](./reference/type-unbound-method.md) | 禁止在没有接收者（`this`）的情况下引用方法。 | `nuxt`, `opinionated` |  |
| <span id="type-use-unknown-in-catch-callback-variable"></span>[`type/use-unknown-in-catch-callback-variable`](./reference/type-use-unknown-in-catch-callback-variable.md) | 要求 Promise 的 catch 回调参数使用显式的 unknown 注解。 | `nuxt`, `opinionated` |  |

## HTML 规则 (9)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="html-deprecated-attr"></span>[`html/deprecated-attr`](./reference/html-deprecated-attr.md) | 禁止已弃用的 HTML 属性。 | `happy-path` |  |
| <span id="html-deprecated-element"></span>[`html/deprecated-element`](./reference/html-deprecated-element.md) | 禁止已弃用的 HTML 元素。 | `happy-path` |  |
| <span id="html-id-duplication"></span>[`html/id-duplication`](./reference/html-id-duplication.md) | 禁止重复的元素 ID。 | `essential` |  |
| <span id="html-no-consecutive-br"></span>[`html/no-consecutive-br`](./reference/html-no-consecutive-br.md) | 禁止连续的 &lt;br&gt; 元素。 | `happy-path` |  |
| <span id="html-no-dupe-style-properties"></span>[`html/no-dupe-style-properties`](./reference/html-no-dupe-style-properties.md) | 禁止内联 style 属性中重复的属性声明。 | `nuxt`, `opinionated` |  |
| <span id="html-no-duplicate-class"></span>[`html/no-duplicate-class`](./reference/html-no-duplicate-class.md) | 禁止静态 class 属性中重复的类名。 | `happy-path` |  |
| <span id="html-no-duplicate-dt"></span>[`html/no-duplicate-dt`](./reference/html-no-duplicate-dt.md) | 禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称。 | `happy-path` |  |
| <span id="html-no-empty-palpable-content"></span>[`html/no-empty-palpable-content`](./reference/html-no-empty-palpable-content.md) | 禁止预期具有可见内容的空元素。 | `happy-path` |  |
| <span id="html-require-datetime"></span>[`html/require-datetime`](./reference/html-require-datetime.md) | 要求 &lt;time&gt; 元素具有 datetime 属性。 | `happy-path` |  |

## 无障碍规则 (31)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="a11y-alt-text"></span>[`a11y/alt-text`](./reference/a11y-alt-text.md) | 要求媒体元素提供替代文本。 | `happy-path` |  |
| <span id="a11y-anchor-has-content"></span>[`a11y/anchor-has-content`](./reference/a11y-anchor-has-content.md) | 要求锚点元素具有可访问的内容。 | `happy-path` |  |
| <span id="a11y-anchor-is-valid"></span>[`a11y/anchor-is-valid`](./reference/a11y-anchor-is-valid.md) | 要求锚点元素具有有效的 href。 | `happy-path` |  |
| <span id="a11y-aria-props"></span>[`a11y/aria-props`](./reference/a11y-aria-props.md) | 禁止无效的 ARIA 属性。 | `happy-path` |  |
| <span id="a11y-aria-role"></span>[`a11y/aria-role`](./reference/a11y-aria-role.md) | 具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色。 | `happy-path` |  |
| <span id="a11y-aria-unsupported-elements"></span>[`a11y/aria-unsupported-elements`](./reference/a11y-aria-unsupported-elements.md) | 禁止在不支持 ARIA 的元素上使用 ARIA 属性。 | `happy-path` |  |
| <span id="a11y-click-events-have-key-events"></span>[`a11y/click-events-have-key-events`](./reference/a11y-click-events-have-key-events.md) | 要求点击事件配有键盘事件处理器。 | `happy-path` |  |
| <span id="a11y-form-control-has-label"></span>[`a11y/form-control-has-label`](./reference/a11y-form-control-has-label.md) | 要求表单控件具有相关联的标签。 | `happy-path` |  |
| <span id="a11y-heading-has-content"></span>[`a11y/heading-has-content`](./reference/a11y-heading-has-content.md) | 要求标题元素具有可访问的内容。 | `happy-path` |  |
| <span id="a11y-heading-levels"></span>[`a11y/heading-levels`](./reference/a11y-heading-levels.md) | 禁止跳过标题层级。 | `nuxt`, `opinionated` |  |
| <span id="a11y-iframe-has-title"></span>[`a11y/iframe-has-title`](./reference/a11y-iframe-has-title.md) | 要求 iframe 元素具有 title 属性。 | `happy-path` |  |
| <span id="a11y-img-alt"></span>[`a11y/img-alt`](./reference/a11y-img-alt.md) | 要求图片提供 alt 属性，以支持无障碍访问。 | — |  |
| <span id="a11y-interactive-supports-focus"></span>[`a11y/interactive-supports-focus`](./reference/a11y-interactive-supports-focus.md) | 要求具有交互角色的元素可获得焦点。 | `happy-path` |  |
| <span id="a11y-label-has-for"></span>[`a11y/label-has-for`](./reference/a11y-label-has-for.md) | 要求标签具有相关联的表单控件。 | `happy-path` |  |
| <span id="a11y-landmark-roles"></span>[`a11y/landmark-roles`](./reference/a11y-landmark-roles.md) | 验证地标角色的位置及唯一性。 | `nuxt`, `opinionated` |  |
| <span id="a11y-media-has-caption"></span>[`a11y/media-has-caption`](./reference/a11y-media-has-caption.md) | 要求媒体元素具有字幕。 | `happy-path` |  |
| <span id="a11y-mouse-events-have-key-events"></span>[`a11y/mouse-events-have-key-events`](./reference/a11y-mouse-events-have-key-events.md) | 要求鼠标事件配有 focus/blur 事件。 | `happy-path` |  |
| <span id="a11y-no-access-key"></span>[`a11y/no-access-key`](./reference/a11y-no-access-key.md) | 禁止使用 accesskey 属性。 | `happy-path` |  |
| <span id="a11y-no-aria-hidden-on-focusable"></span>[`a11y/no-aria-hidden-on-focusable`](./reference/a11y-no-aria-hidden-on-focusable.md) | 禁止在可获得焦点的元素上使用 aria-hidden="true"。 | `happy-path` |  |
| <span id="a11y-no-autofocus"></span>[`a11y/no-autofocus`](./reference/a11y-no-autofocus.md) | 禁止使用 autofocus 属性。 | `happy-path` |  |
| <span id="a11y-no-distracting-elements"></span>[`a11y/no-distracting-elements`](./reference/a11y-no-distracting-elements.md) | 禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素。 | `happy-path` |  |
| <span id="a11y-no-i-for-icon"></span>[`a11y/no-i-for-icon`](./reference/a11y-no-i-for-icon.md) | 禁止使用 &lt;i&gt; 元素表示图标。 | `happy-path` |  |
| <span id="a11y-no-redundant-roles"></span>[`a11y/no-redundant-roles`](./reference/a11y-no-redundant-roles.md) | 禁止多余的 ARIA 角色。 | `happy-path` | ✓ |
| <span id="a11y-no-refer-to-non-existent-id"></span>[`a11y/no-refer-to-non-existent-id`](./reference/a11y-no-refer-to-non-existent-id.md) | 禁止引用不存在的 ID。 | `happy-path` |  |
| <span id="a11y-no-role-presentation-on-focusable"></span>[`a11y/no-role-presentation-on-focusable`](./reference/a11y-no-role-presentation-on-focusable.md) | 禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"。 | `happy-path` |  |
| <span id="a11y-no-static-element-interactions"></span>[`a11y/no-static-element-interactions`](./reference/a11y-no-static-element-interactions.md) | 禁止在静态元素上设置事件处理器。 | `happy-path` |  |
| <span id="a11y-placeholder-label-option"></span>[`a11y/placeholder-label-option`](./reference/a11y-placeholder-label-option.md) | 要求 required 的 select 带有占位选项。 | `nuxt`, `opinionated` |  |
| <span id="a11y-role-has-required-aria-props"></span>[`a11y/role-has-required-aria-props`](./reference/a11y-role-has-required-aria-props.md) | 要求 ARIA 角色具有必需属性。 | `happy-path` |  |
| <span id="a11y-role-img-has-name"></span>[`a11y/role-img-has-name`](./reference/a11y-role-img-has-name.md) | 要求 role="img" 的元素具有无障碍名称。 | — |  |
| <span id="a11y-tabindex-no-positive"></span>[`a11y/tabindex-no-positive`](./reference/a11y-tabindex-no-positive.md) | 禁止正数 tabindex 值。 | `happy-path` |  |
| <span id="a11y-use-list"></span>[`a11y/use-list`](./reference/a11y-use-list.md) | 建议为类似项目符号的文本使用列表元素。 | `nuxt`, `opinionated` |  |

## SSR 规则 (2)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="ssr-no-browser-globals-in-ssr"></span>[`ssr/no-browser-globals-in-ssr`](./reference/ssr-no-browser-globals-in-ssr.md) | 禁止 SSR 上下文中的浏览器专用全局变量。 | `happy-path` |  |
| <span id="ssr-no-hydration-mismatch"></span>[`ssr/no-hydration-mismatch`](./reference/ssr-no-hydration-mismatch.md) | 禁止导致水合不匹配的不确定值。 | `happy-path` |  |

## Vapor 规则 (4)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="vapor-no-inline-template"></span>[`vapor/no-inline-template`](./reference/vapor-no-inline-template.md) | 禁止已弃用的 inline-template 属性。 | `nuxt`, `opinionated` |  |
| <span id="vapor-no-vue-lifecycle-events"></span>[`vapor/no-vue-lifecycle-events`](./reference/vapor-no-vue-lifecycle-events.md) | 禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）。 | `happy-path` |  |
| <span id="vapor-prefer-static-class"></span>[`vapor/prefer-static-class`](./reference/vapor-prefer-static-class.md) | 字符串字面量优先使用静态 class，而不是动态 class 绑定。 | `nuxt`, `opinionated` | ✓ |
| <span id="vapor-require-vapor-attribute"></span>[`vapor/require-vapor-attribute`](./reference/vapor-require-vapor-attribute.md) | 建议为 script setup 添加 vapor 属性。 | `nuxt`, `opinionated` |  |

## 生态系统与 Nuxt 规则 (13)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="ecosystem-nuxt-prefer-nuxt-link"></span>[`ecosystem/nuxt-prefer-nuxt-link`](./reference/ecosystem-nuxt-prefer-nuxt-link.md) | 应用内部链接优先使用 NuxtLink。 | `nuxt` |  |
| <span id="ecosystem-pinia-prefer-store-to-refs"></span>[`ecosystem/pinia-prefer-store-to-refs`](./reference/ecosystem-pinia-prefer-store-to-refs.md) | 解构 Pinia store 时优先使用 storeToRefs()。 | `ecosystem` |  |
| <span id="ecosystem-router-link-require-to"></span>[`ecosystem/router-link-require-to`](./reference/ecosystem-router-link-require-to.md) | 要求 RouterLink 和 NuxtLink 组件具有 `to` 目标。 | `ecosystem` |  |
| <span id="ecosystem-void-link-require-href"></span>[`ecosystem/void-link-require-href`](./reference/ecosystem-void-link-require-href.md) | 要求 Void Vue Link 组件具有 `href`。 | `ecosystem` |  |
| <span id="ecosystem-void-link-valid-method"></span>[`ecosystem/void-link-valid-method`](./reference/ecosystem-void-link-valid-method.md) | 验证 Void Vue Link 的静态 method prop。 | `ecosystem` |  |
| <span id="ecosystem-vue-i18n-no-missing-key"></span>[`ecosystem/vue-i18n-no-missing-key`](./reference/ecosystem-vue-i18n-no-missing-key.md) | 报告本地 SFC 消息中不存在的静态 vue-i18n 键。 | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-link"></span>[`ecosystem/vue-router-prefer-named-link`](./reference/ecosystem-vue-router-prefer-named-link.md) | RouterLink 优先使用具名路由对象，而不是静态路径字符串。 | `ecosystem` |  |
| <span id="ecosystem-vue-router-prefer-named-push"></span>[`ecosystem/vue-router-prefer-named-push`](./reference/ecosystem-vue-router-prefer-named-push.md) | Vue Router 编程式导航优先使用具名路由对象。 | `ecosystem` |  |
| <span id="ecosystem-vue-test-utils-no-html-snapshot"></span>[`ecosystem/vue-test-utils-no-html-snapshot`](./reference/ecosystem-vue-test-utils-no-html-snapshot.md) | 避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照。 | `ecosystem` |  |
| <span id="nuxt-no-nuxt-config-test-key"></span>[`nuxt/no-nuxt-config-test-key`](./reference/nuxt-no-nuxt-config-test-key.md) | 禁止在 Nuxt 配置中设置 `test` 键。 | `nuxt` |  |
| <span id="nuxt-no-page-meta-runtime-values"></span>[`nuxt/no-page-meta-runtime-values`](./reference/nuxt-no-page-meta-runtime-values.md) | 禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行。 | `nuxt` |  |
| <span id="nuxt-nuxt-config-keys-order"></span>[`nuxt/nuxt-config-keys-order`](./reference/nuxt-nuxt-config-keys-order.md) | 优先使用推荐的 Nuxt 配置属性顺序。 | `nuxt` | ✓ |
| <span id="nuxt-prefer-import-meta"></span>[`nuxt/prefer-import-meta`](./reference/nuxt-prefer-import-meta.md) | 优先使用 `import.meta.*`，而不是 `process.*`。 | `nuxt` | ✓ |

## CSS 规则 (10)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="css-no-display-none"></span>[`css/no-display-none`](./reference/css-no-display-none.md) | 建议使用 v-show 代替 display: none。 | `opinionated`, `nuxt` |  |
| <span id="css-no-hardcoded-values"></span>[`css/no-hardcoded-values`](./reference/css-no-hardcoded-values.md) | 建议使用 CSS 变量代替硬编码值。 | `opinionated`, `nuxt` |  |
| <span id="css-no-id-selectors"></span>[`css/no-id-selectors`](./reference/css-no-id-selectors.md) | 不建议在 CSS 中使用 ID 选择器。 | `opinionated`, `nuxt` |  |
| <span id="css-no-important"></span>[`css/no-important`](./reference/css-no-important.md) | 不建议在 CSS 中使用 !important。 | `opinionated`, `nuxt` |  |
| <span id="css-no-utility-classes"></span>[`css/no-utility-classes`](./reference/css-no-utility-classes.md) | 警告在组件样式中实现工具类。 | `opinionated`, `nuxt` |  |
| <span id="css-no-v-bind-performance"></span>[`css/no-v-bind-performance`](./reference/css-no-v-bind-performance.md) | 警告 CSS v-bind() 的性能开销。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-logical-properties"></span>[`css/prefer-logical-properties`](./reference/css-prefer-logical-properties.md) | 建议使用 CSS 逻辑属性，以更好地支持国际化。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-nested-selectors"></span>[`css/prefer-nested-selectors`](./reference/css-prefer-nested-selectors.md) | 建议为后代选择器使用 CSS 嵌套。 | `opinionated`, `nuxt` |  |
| <span id="css-prefer-slotted"></span>[`css/prefer-slotted`](./reference/css-prefer-slotted.md) | 建议使用 ::v-slotted() 为插槽内容设置样式。 | `opinionated`, `nuxt` |  |
| <span id="css-require-font-display"></span>[`css/require-font-display`](./reference/css-require-font-display.md) | 要求 @font-face 规则具有 font-display。 | `opinionated`, `nuxt` |  |

## Musea 规则 (6)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="musea-no-empty-variant"></span>[`musea/no-empty-variant`](./reference/musea-no-empty-variant.md) | 禁止空的 &lt;variant&gt; 块。 | — |  |
| <span id="musea-prefer-design-tokens"></span>[`musea/prefer-design-tokens`](./reference/musea-prefer-design-tokens.md) | 优先使用设计令牌 CSS 变量，而不是硬编码的原始值。 | — |  |
| <span id="musea-require-component"></span>[`musea/require-component`](./reference/musea-require-component.md) | 要求 &lt;art&gt; 块具有 component 属性。 | — |  |
| <span id="musea-require-title"></span>[`musea/require-title`](./reference/musea-require-title.md) | 要求 &lt;art&gt; 块具有 title 属性。 | — |  |
| <span id="musea-unique-variant-names"></span>[`musea/unique-variant-names`](./reference/musea-unique-variant-names.md) | 要求 variant 名称唯一。 | — |  |
| <span id="musea-valid-variant"></span>[`musea/valid-variant`](./reference/musea-valid-variant.md) | 要求 &lt;variant&gt; 块具有 name 属性。 | — |  |

## petite-vue 规则 (3)

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="petite-vue-no-unsupported-directive"></span>[`petite-vue/no-unsupported-directive`](./reference/petite-vue-no-unsupported-directive.md) | 禁止 petite-vue 不支持的指令。 | — |  |
| <span id="petite-vue-valid-v-effect"></span>[`petite-vue/valid-v-effect`](./reference/petite-vue-valid-v-effect.md) | 要求 v-effect 具有非空表达式。 | — |  |
| <span id="petite-vue-valid-v-scope"></span>[`petite-vue/valid-v-scope`](./reference/petite-vue-valid-v-scope.md) | 要求 v-scope 绑定对象字面量。 | — |  |

## 跨文件规则 (66)

在传给 `withVue` 的 `linter` 中设置 `crossFile: true`，或运行 `vize lint --cross-file` 来启用这些检查。诊断显示为 `vize:croquis/cf/<name>`；在 `linter.rules` 中去掉 `vize:`，写作 `croquis/cf/<name>`。

| 规则 | 检查内容 | 状态 |
| --- | --- | --- |
| <span id="ecosystem-vue-router-extra-param"></span>[`ecosystem/vue-router-extra-param`](./project/ecosystem-vue-router-extra-param.md) | 报告目标路由未声明的导航参数。 | 由 `vp run lint` 报告 |
| <span id="ecosystem-vue-router-missing-param"></span>[`ecosystem/vue-router-missing-param`](./project/ecosystem-vue-router-missing-param.md) | 报告具名导航省略了必需的路由参数。 | 由 `vp run lint` 报告 |
| <span id="ecosystem-vue-router-param-type"></span>[`ecosystem/vue-router-param-type`](./project/ecosystem-vue-router-param-type.md) | 报告路由参数值不被所声明的路径段接受。 | 由 `vp run lint` 报告 |
| <span id="ecosystem-vue-router-unknown-route"></span>[`ecosystem/vue-router-unknown-route`](./project/ecosystem-vue-router-unknown-route.md) | 报告已安装路由器未声明的导航名称。 | 由 `vp run lint` 报告 |
| <span id="html-cross-component-nesting"></span>[`html/cross-component-nesting`](./project/html-cross-component-nesting.md) | 检查导入组件组合后的实际 HTML 嵌套。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-array-mutation"></span>[`vize:croquis/cf/array-mutation`](./project/vize-croquis-cf-array-mutation.md) | 在 Vue 2.7 中，使用可通知更新的操作替换数组索引处的值。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-async-boundary"></span>[`vize:croquis/cf/async-boundary`](./project/vize-croquis-cf-async-boundary.md) | 响应式状态跨越异步边界，可能被观察为过期值。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-async-no-suspense"></span>[`vize:croquis/cf/async-no-suspense`](./project/vize-croquis-cf-async-no-suspense.md) | 异步组件在没有 Suspense 边界的情况下渲染。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-browser-api-ssr"></span>[`vize:croquis/cf/browser-api-ssr`](./project/vize-croquis-cf-browser-api-ssr.md) | 在组件可能于服务端渲染的位置使用了浏览器专用 API。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-circular-dep"></span>[`vize:croquis/cf/circular-dep`](./project/vize-croquis-cf-circular-dep.md) | 组件相互导入，形成循环。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-circular-reactive-dependency"></span>[`vize:croquis/cf/circular-reactive-dependency`](./project/vize-croquis-cf-circular-reactive-dependency.md) | 响应式计算相互依赖，形成循环。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-closure-captures-reactive"></span>[`vize:croquis/cf/closure-captures-reactive`](./project/vize-croquis-cf-closure-captures-reactive.md) | 闭包捕获了响应式值，无法看到后续更新。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-composable-outside-setup"></span>[`vize:croquis/cf/composable-outside-setup`](./project/vize-croquis-cf-composable-outside-setup.md) | 组合式函数在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-computed-side-effects"></span>[`vize:croquis/cf/computed-side-effects`](./project/vize-croquis-cf-computed-side-effects.md) | 计算 getter 写入状态或执行其他副作用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-deep-import"></span>[`vize:croquis/cf/deep-import`](./project/vize-croquis-cf-deep-import.md) | 导入链超过项目允许的深度。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-destructuring-breaks-reactivity"></span>[`vize:croquis/cf/destructuring-breaks-reactivity`](./project/vize-croquis-cf-destructuring-breaks-reactivity.md) | 解构响应式对象会复制字段并丢失跟踪。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-di-outside-setup"></span>[`vize:croquis/cf/di-outside-setup`](./project/vize-croquis-cf-di-outside-setup.md) | `provide` 或 `inject` 在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-dom-access-without-next-tick"></span>[`vize:croquis/cf/dom-access-without-next-tick`](./project/vize-croquis-cf-dom-access-without-next-tick.md) | 在 Vue 完成更新刷新之前读取 DOM。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-duplicate-id"></span>[`vize:croquis/cf/duplicate-id`](./project/vize-croquis-cf-duplicate-id.md) | 多个组件使用同一个元素 id。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-event-listener-leak"></span>[`vize:croquis/cf/event-listener-leak`](./project/vize-croquis-cf-event-listener-leak.md) | 注册了事件监听器，却从未移除。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-event-modifier"></span>[`vize:croquis/cf/event-modifier`](./project/vize-croquis-cf-event-modifier.md) | 事件监听器使用了发出事件不支持的修饰符。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-hydration-risk"></span>[`vize:croquis/cf/hydration-risk`](./project/vize-croquis-cf-hydration-risk.md) | 服务器与客户端的渲染结果可能不一致。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-inherit-attrs-unused"></span>[`vize:croquis/cf/inherit-attrs-unused`](./project/vize-croquis-cf-inherit-attrs-unused.md) | 设置了 `inheritAttrs: false`，但组件从不读取属性。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-inject-without-symbol"></span>[`vize:croquis/cf/inject-without-symbol`](./project/vize-croquis-cf-inject-without-symbol.md) | `inject` 使用普通键，而不是 `InjectionKey` symbol。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-injected-async-mutation-race"></span>[`vize:croquis/cf/injected-async-mutation-race`](./project/vize-croquis-cf-injected-async-mutation-race.md) | 注入值被可能发生竞态的异步任务修改。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-lifecycle-outside-setup"></span>[`vize:croquis/cf/lifecycle-outside-setup`](./project/vize-croquis-cf-lifecycle-outside-setup.md) | 生命周期钩子在 `setup` 外注册。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-lifecycle-without-cleanup"></span>[`vize:croquis/cf/lifecycle-without-cleanup`](./project/vize-croquis-cf-lifecycle-without-cleanup.md) | 生命周期钩子启动任务，却从不清理。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-missing-required-prop"></span>[`vize:croquis/cf/missing-required-prop`](./project/vize-croquis-cf-missing-required-prop.md) | 未传入必需的 prop。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-missing-suspense"></span>[`vize:croquis/cf/missing-suspense`](./project/vize-croquis-cf-missing-suspense.md) | 异步依赖在 Suspense 边界之外使用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-module-scope-reactive"></span>[`vize:croquis/cf/module-scope-reactive`](./project/vize-croquis-cf-module-scope-reactive.md) | 响应式状态在模块作用域创建，被所有调用者共享。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-multi-root-attrs"></span>[`vize:croquis/cf/multi-root-attrs`](./project/vize-croquis-cf-multi-root-attrs.md) | 多根节点组件接收了属性，却没有放置目标。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-mutated-after-escape"></span>[`vize:croquis/cf/mutated-after-escape`](./project/vize-croquis-cf-mutated-after-escape.md) | 响应式对象逸出其所有者后被修改。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-non-reactive-provide"></span>[`vize:croquis/cf/non-reactive-provide`](./project/vize-croquis-cf-non-reactive-provide.md) | 提供的值不具响应性，后代无法看到更新。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-non-unique-id"></span>[`vize:croquis/cf/non-unique-id`](./project/vize-croquis-cf-non-unique-id.md) | 循环内的元素 id 对每个列表项并不唯一。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-object-identity-comparison"></span>[`vize:croquis/cf/object-identity-comparison`](./project/vize-croquis-cf-object-identity-comparison.md) | 用稳定的记录 ID 比较原始对象与响应式代理所表示的记录。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-pinia-getter"></span>[`vize:croquis/cf/pinia-getter`](./project/vize-croquis-cf-pinia-getter.md) | 避免在 setup 中将 Pinia getter 的值复制为不再跟随更新的快照。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-prop-type-mismatch"></span>[`vize:croquis/cf/prop-type-mismatch`](./project/vize-croquis-cf-prop-type-mismatch.md) | 传入的 prop 值与声明类型不匹配。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-provide-inject-type"></span>[`vize:croquis/cf/provide-inject-type`](./project/vize-croquis-cf-provide-inject-type.md) | 提供值与其 inject 类型不一致。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-provide-without-symbol"></span>[`vize:croquis/cf/provide-without-symbol`](./project/vize-croquis-cf-provide-without-symbol.md) | `provide` 使用普通键，而不是 `InjectionKey` symbol。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-reactive-export"></span>[`vize:croquis/cf/reactive-export`](./project/vize-croquis-cf-reactive-export.md) | 响应式状态从模块导出。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-reactivity-outside-setup"></span>[`vize:croquis/cf/reactivity-outside-setup`](./project/vize-croquis-cf-reactivity-outside-setup.md) | 响应式 API 在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-reassignment-breaks-reactivity"></span>[`vize:croquis/cf/reassignment-breaks-reactivity`](./project/vize-croquis-cf-reassignment-breaks-reactivity.md) | 重新赋值响应式绑定，将其替换为普通值。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-reference-escapes-scope"></span>[`vize:croquis/cf/reference-escapes-scope`](./project/vize-croquis-cf-reference-escapes-scope.md) | 响应式引用逸出拥有其生命周期的作用域。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-setup-context-violation"></span>[`vize:croquis/cf/setup-context-violation`](./project/vize-croquis-cf-setup-context-violation.md) | 在每个实例的 setup 中创建应由组件实例独立持有的响应式状态。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-shallow-deep-access"></span>[`vize:croquis/cf/shallow-deep-access`](./project/vize-croquis-cf-shallow-deep-access.md) | 读取 `shallowReactive` 或 `shallowRef` 的深层属性，却假定它受到跟踪。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-spread-breaks-reactivity"></span>[`vize:croquis/cf/spread-breaks-reactivity`](./project/vize-croquis-cf-spread-breaks-reactivity.md) | 展开响应式对象会复制值并丢失跟踪。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-suspense-no-fallback"></span>[`vize:croquis/cf/suspense-no-fallback`](./project/vize-croquis-cf-suspense-no-fallback.md) | `<Suspense>` 没有后备内容。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-template-ref-timing"></span>[`vize:croquis/cf/template-ref-timing`](./project/vize-croquis-cf-template-ref-timing.md) | 模板 ref 在组件挂载之前被读取。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-toraw-mutation"></span>[`vize:croquis/cf/toraw-mutation`](./project/vize-croquis-cf-toraw-mutation.md) | 使用 `toRaw` 后修改了原始对象。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-uncaught-error"></span>[`vize:croquis/cf/uncaught-error`](./project/vize-croquis-cf-uncaught-error.md) | 组件可能抛错，却没有错误边界捕获。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-undeclared-emit"></span>[`vize:croquis/cf/undeclared-emit`](./project/vize-croquis-cf-undeclared-emit.md) | 组件发出了未声明的事件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-undeclared-prop"></span>[`vize:croquis/cf/undeclared-prop`](./project/vize-croquis-cf-undeclared-prop.md) | 父组件传入了子组件未声明的 prop。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-undefined-slot"></span>[`vize:croquis/cf/undefined-slot`](./project/vize-croquis-cf-undefined-slot.md) | 父组件填充了子组件未暴露的插槽。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-unhandled-event"></span>[`vize:croquis/cf/unhandled-event`](./project/vize-croquis-cf-unhandled-event.md) | 子组件发出的事件没有父组件处理。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unmatched-inject"></span>[`vize:croquis/cf/unmatched-inject`](./project/vize-croquis-cf-unmatched-inject.md) | `inject` 指定的键没有任何祖先提供。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-unmatched-listener"></span>[`vize:croquis/cf/unmatched-listener`](./project/vize-croquis-cf-unmatched-listener.md) | 父组件监听了子组件不发出的事件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unregistered-component"></span>[`vize:croquis/cf/unregistered-component`](./project/vize-croquis-cf-unregistered-component.md) | 模板使用了未注册或导入的组件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unresolved-import"></span>[`vize:croquis/cf/unresolved-import`](./project/vize-croquis-cf-unresolved-import.md) | 导入无法解析为模块。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unused-attrs"></span>[`vize:croquis/cf/unused-attrs`](./project/vize-croquis-cf-unused-attrs.md) | 透传属性传给了未使用它们的多根节点组件。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unused-emit"></span>[`vize:croquis/cf/unused-emit`](./project/vize-croquis-cf-unused-emit.md) | 已声明的 emit 从未使用。 | 实验性；CLI 尚未报告 |
| <span id="vize-croquis-cf-unused-provide"></span>[`vize:croquis/cf/unused-provide`](./project/vize-croquis-cf-unused-provide.md) | 提供的键从未被注入。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-value-extraction-breaks-reactivity"></span>[`vize:croquis/cf/value-extraction-breaks-reactivity`](./project/vize-croquis-cf-value-extraction-breaks-reactivity.md) | 将响应式值读取到局部变量后丢失后续更新。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-watch-can-be-computed"></span>[`vize:croquis/cf/watch-can-be-computed`](./project/vize-croquis-cf-watch-can-be-computed.md) | 侦听器只将值复制到状态，可以改为计算属性。 | 计划中；尚未报告 |
| <span id="vize-croquis-cf-watcheffect-async"></span>[`vize:croquis/cf/watcheffect-async`](./project/vize-croquis-cf-watcheffect-async.md) | 为异步 watchEffect 处理失效清理，避免写入过期响应。 | 由 `vp run lint` 报告 |
| <span id="vize-croquis-cf-watcher-outside-setup"></span>[`vize:croquis/cf/watcher-outside-setup`](./project/vize-croquis-cf-watcher-outside-setup.md) | `watch` 或 `watchEffect` 在 `setup` 外被调用。 | 计划中；尚未报告 |
| <span id="vue-cross-file-attrs-fallthrough"></span>[`vue/cross-file-attrs-fallthrough`](./project/vue-cross-file-attrs-fallthrough.md) | 父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。 | 由 `vp run lint` 报告 |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
