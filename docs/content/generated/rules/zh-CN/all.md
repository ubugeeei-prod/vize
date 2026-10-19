---
title: "全部规则"
---

# 全部规则

通过此索引查找规则的用途、默认配置及所属分组。分组页面包含说明、配置、完整的错误和正确示例，以及当前支持范围。此页面现有的规则和示例链接仍然可用。

66 个项目条目链接到跨文件规则页面中的完整共享文件以及错误和正确示例。60 个分析器代码保留其实际支持边界：19 个 CLI 代码（18 对已验证的源代码示例，以及一个保留响应式图的说明性项目）、16 个该 CLI 阶段不会单独输出的实验性 Rust 分析器代码，以及 25 个当前没有生成器的契约。配置 ID 不会启用尚不可用的生成器。

公共 CLI 可通过 `vize lint --cross-file` 执行同一检查。显示的 `vize:croquis/cf/*` 代码在 `vue.linter.rules` 中写为 `croquis/cf/*`（省略 `vize:`）。信息和提示诊断会转换为 CLI 警告。相关位置说明来源与消费者之间的关系。

<span id="所有铜绿统治"></span>
<span id="分类"></span>
<span id="必看-48"></span>
<span id="强烈推荐-12"></span>
<span id="推荐-42"></span>
<span id="无障碍性-31"></span>
<span id="html-符合性-9"></span>
<span id="类型感知-5"></span>
<span id="蒸汽-7"></span>
<span id="生态系统-9"></span>
<span id="css-10"></span>
<span id="博物馆-6"></span>
<span id="剧本-60"></span>
<span id="essential-48"></span>
<span id="strongly-recommended-12"></span>
<span id="recommended-42"></span>
<span id="accessibility-31"></span>
<span id="html-conformance-9"></span>
<span id="type-aware-5"></span>
<span id="vapor-7"></span>
<span id="ecosystem-9"></span>
<span id="musea-6"></span>
<span id="script-60"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`a11y/alt-text`](#a11y-alt-text) | [错误示例](#a11y-alt-text-bad) · [正确示例](#a11y-alt-text-good) | 要求媒体元素提供替代文本 |
| [`a11y/anchor-has-content`](#a11y-anchor-has-content) | [错误示例](#a11y-anchor-has-content-bad) · [正确示例](#a11y-anchor-has-content-good) | 要求锚点元素具有可访问的内容 |
| [`a11y/anchor-is-valid`](#a11y-anchor-is-valid) | [错误示例](#a11y-anchor-is-valid-bad) · [正确示例](#a11y-anchor-is-valid-good) | 要求锚点元素具有有效的 href |
| [`a11y/aria-props`](#a11y-aria-props) | [错误示例](#a11y-aria-props-bad) · [正确示例](#a11y-aria-props-good) | 禁止无效的 ARIA 属性 |
| [`a11y/aria-role`](#a11y-aria-role) | [错误示例](#a11y-aria-role-bad) · [正确示例](#a11y-aria-role-good) | 具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色 |
| [`a11y/aria-unsupported-elements`](#a11y-aria-unsupported-elements) | [错误示例](#a11y-aria-unsupported-elements-bad) · [正确示例](#a11y-aria-unsupported-elements-good) | 禁止在不支持 ARIA 的元素上使用 ARIA 属性 |
| [`a11y/click-events-have-key-events`](#a11y-click-events-have-key-events) | [错误示例](#a11y-click-events-have-key-events-bad) · [正确示例](#a11y-click-events-have-key-events-good) | 要求点击事件配有键盘事件处理器 |
| [`a11y/form-control-has-label`](#a11y-form-control-has-label) | [错误示例](#a11y-form-control-has-label-bad) · [正确示例](#a11y-form-control-has-label-good) | 要求表单控件具有相关联的标签 |
| [`a11y/heading-has-content`](#a11y-heading-has-content) | [错误示例](#a11y-heading-has-content-bad) · [正确示例](#a11y-heading-has-content-good) | 要求标题元素具有可访问的内容 |
| [`a11y/heading-levels`](#a11y-heading-levels) | [错误示例](#a11y-heading-levels-bad) · [正确示例](#a11y-heading-levels-good) | 禁止跳过标题层级 |
| [`a11y/iframe-has-title`](#a11y-iframe-has-title) | [错误示例](#a11y-iframe-has-title-bad) · [正确示例](#a11y-iframe-has-title-good) | 要求 iframe 元素具有 title 属性 |
| [`a11y/img-alt`](#a11y-img-alt) | [错误示例](#a11y-img-alt-bad) · [正确示例](#a11y-img-alt-good) | 要求图片提供 alt 属性，以支持无障碍访问 |
| [`a11y/interactive-supports-focus`](#a11y-interactive-supports-focus) | [错误示例](#a11y-interactive-supports-focus-bad) · [正确示例](#a11y-interactive-supports-focus-good) | 要求具有交互角色的元素可获得焦点 |
| [`a11y/label-has-for`](#a11y-label-has-for) | [错误示例](#a11y-label-has-for-bad) · [正确示例](#a11y-label-has-for-good) | 要求标签具有相关联的表单控件 |
| [`a11y/landmark-roles`](#a11y-landmark-roles) | [错误示例](#a11y-landmark-roles-bad) · [正确示例](#a11y-landmark-roles-good) | 验证地标角色的位置及唯一性 |
| [`a11y/media-has-caption`](#a11y-media-has-caption) | [错误示例](#a11y-media-has-caption-bad) · [正确示例](#a11y-media-has-caption-good) | 要求媒体元素具有字幕 |
| [`a11y/mouse-events-have-key-events`](#a11y-mouse-events-have-key-events) | [错误示例](#a11y-mouse-events-have-key-events-bad) · [正确示例](#a11y-mouse-events-have-key-events-good) | 要求鼠标事件配有 focus/blur 事件 |
| [`a11y/no-access-key`](#a11y-no-access-key) | [错误示例](#a11y-no-access-key-bad) · [正确示例](#a11y-no-access-key-good) | 禁止使用 accesskey 属性 |
| [`a11y/no-aria-hidden-on-focusable`](#a11y-no-aria-hidden-on-focusable) | [错误示例](#a11y-no-aria-hidden-on-focusable-bad) · [正确示例](#a11y-no-aria-hidden-on-focusable-good) | 禁止在可获得焦点的元素上使用 aria-hidden="true" |
| [`a11y/no-autofocus`](#a11y-no-autofocus) | [错误示例](#a11y-no-autofocus-bad) · [正确示例](#a11y-no-autofocus-good) | 禁止使用 autofocus 属性 |
| [`a11y/no-distracting-elements`](#a11y-no-distracting-elements) | [错误示例](#a11y-no-distracting-elements-bad) · [正确示例](#a11y-no-distracting-elements-good) | 禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素 |
| [`a11y/no-i-for-icon`](#a11y-no-i-for-icon) | [错误示例](#a11y-no-i-for-icon-bad) · [正确示例](#a11y-no-i-for-icon-good) | 禁止使用 &lt;i&gt; 元素表示图标 |
| [`a11y/no-redundant-roles`](#a11y-no-redundant-roles) | [错误示例](#a11y-no-redundant-roles-bad) · [正确示例](#a11y-no-redundant-roles-good) | 禁止多余的 ARIA 角色 |
| [`a11y/no-refer-to-non-existent-id`](#a11y-no-refer-to-non-existent-id) | [错误示例](#a11y-no-refer-to-non-existent-id-bad) · [正确示例](#a11y-no-refer-to-non-existent-id-good) | 禁止引用不存在的 ID |
| [`a11y/no-role-presentation-on-focusable`](#a11y-no-role-presentation-on-focusable) | [错误示例](#a11y-no-role-presentation-on-focusable-bad) · [正确示例](#a11y-no-role-presentation-on-focusable-good) | 禁止在可获得焦点的元素上使用 role="presentation" 或 role="none" |
| [`a11y/no-static-element-interactions`](#a11y-no-static-element-interactions) | [错误示例](#a11y-no-static-element-interactions-bad) · [正确示例](#a11y-no-static-element-interactions-good) | 禁止在静态元素上设置事件处理器 |
| [`a11y/placeholder-label-option`](#a11y-placeholder-label-option) | [错误示例](#a11y-placeholder-label-option-bad) · [正确示例](#a11y-placeholder-label-option-good) | 要求 select 的占位选项具有 disabled 或 hidden |
| [`a11y/role-has-required-aria-props`](#a11y-role-has-required-aria-props) | [错误示例](#a11y-role-has-required-aria-props-bad) · [正确示例](#a11y-role-has-required-aria-props-good) | 要求 ARIA 角色具有必需属性 |
| [`a11y/tabindex-no-positive`](#a11y-tabindex-no-positive) | [错误示例](#a11y-tabindex-no-positive-bad) · [正确示例](#a11y-tabindex-no-positive-good) | 禁止正数 tabindex 值 |
| [`a11y/use-list`](#a11y-use-list) | [错误示例](#a11y-use-list-bad) · [正确示例](#a11y-use-list-good) | 建议为类似项目符号的文本使用列表元素 |
| [`css/no-display-none`](#css-no-display-none) | [错误示例](#css-no-display-none-bad) · [正确示例](#css-no-display-none-good) | 建议使用 v-show 代替 display: none |
| [`css/no-hardcoded-values`](#css-no-hardcoded-values) | [错误示例](#css-no-hardcoded-values-bad) · [正确示例](#css-no-hardcoded-values-good) | 建议使用 CSS 变量代替硬编码值 |
| [`css/no-id-selectors`](#css-no-id-selectors) | [错误示例](#css-no-id-selectors-bad) · [正确示例](#css-no-id-selectors-good) | 不建议在 CSS 中使用 ID 选择器 |
| [`css/no-important`](#css-no-important) | [错误示例](#css-no-important-bad) · [正确示例](#css-no-important-good) | 不建议在 CSS 中使用 !important |
| [`css/no-utility-classes`](#css-no-utility-classes) | [错误示例](#css-no-utility-classes-bad) · [正确示例](#css-no-utility-classes-good) | 警告在组件样式中实现工具类 |
| [`css/no-v-bind-performance`](#css-no-v-bind-performance) | [错误示例](#css-no-v-bind-performance-bad) · [正确示例](#css-no-v-bind-performance-good) | 警告 CSS v-bind() 的性能开销 |
| [`css/prefer-logical-properties`](#css-prefer-logical-properties) | [错误示例](#css-prefer-logical-properties-bad) · [正确示例](#css-prefer-logical-properties-good) | 建议使用 CSS 逻辑属性，以更好地支持国际化 |
| [`css/prefer-nested-selectors`](#css-prefer-nested-selectors) | [错误示例](#css-prefer-nested-selectors-bad) · [正确示例](#css-prefer-nested-selectors-good) | 建议为后代选择器使用 CSS 嵌套 |
| [`css/prefer-slotted`](#css-prefer-slotted) | [错误示例](#css-prefer-slotted-bad) · [正确示例](#css-prefer-slotted-good) | 建议使用 ::v-slotted() 为插槽内容设置样式 |
| [`css/require-font-display`](#css-require-font-display) | [错误示例](#css-require-font-display-bad) · [正确示例](#css-require-font-display-good) | 要求 @font-face 规则具有 font-display |
| [`ecosystem/nuxt-prefer-nuxt-link`](#ecosystem-nuxt-prefer-nuxt-link) | [错误示例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [正确示例](#ecosystem-nuxt-prefer-nuxt-link-good) | 应用内部链接优先使用 NuxtLink |
| [`ecosystem/pinia-prefer-store-to-refs`](#ecosystem-pinia-prefer-store-to-refs) | [错误示例](#ecosystem-pinia-prefer-store-to-refs-bad) · [正确示例](#ecosystem-pinia-prefer-store-to-refs-good) | 解构 Pinia store 时优先使用 storeToRefs() |
| [`ecosystem/router-link-require-to`](#ecosystem-router-link-require-to) | [错误示例](#ecosystem-router-link-require-to-bad) · [正确示例](#ecosystem-router-link-require-to-good) | 要求 RouterLink 和 NuxtLink 组件具有 `to` 目标 |
| [`ecosystem/void-link-require-href`](#ecosystem-void-link-require-href) | [错误示例](#ecosystem-void-link-require-href-bad) · [正确示例](#ecosystem-void-link-require-href-good) | 要求 Void Vue Link 组件具有 `href` |
| [`ecosystem/void-link-valid-method`](#ecosystem-void-link-valid-method) | [错误示例](#ecosystem-void-link-valid-method-bad) · [正确示例](#ecosystem-void-link-valid-method-good) | 验证 Void Vue Link 的静态 method prop |
| [`ecosystem/vue-i18n-no-missing-key`](#ecosystem-vue-i18n-no-missing-key) | [错误示例](#ecosystem-vue-i18n-no-missing-key-bad) · [正确示例](#ecosystem-vue-i18n-no-missing-key-good) | 报告本地 SFC 消息中不存在的静态 vue-i18n 键 |
| [`ecosystem/vue-router-extra-param`](#ecosystem-vue-router-extra-param) | [错误示例](#ecosystem-vue-router-extra-param-bad) · [正确示例](#ecosystem-vue-router-extra-param-good) | 路由未声明 tab；Vue Router 会丢弃它。 |
| [`ecosystem/vue-router-missing-param`](#ecosystem-vue-router-missing-param) | [错误示例](#ecosystem-vue-router-missing-param-bad) · [正确示例](#ecosystem-vue-router-missing-param-good) | 缺少必需的 postId；依赖当前路由并不稳健。 |
| [`ecosystem/vue-router-param-type`](#ecosystem-vue-router-param-type) | [错误示例](#ecosystem-vue-router-param-type-bad) · [正确示例](#ecosystem-vue-router-param-type-good) | postId 不是可重复参数，因此数组无效。 |
| [`ecosystem/vue-router-prefer-named-link`](#ecosystem-vue-router-prefer-named-link) | [错误示例](#ecosystem-vue-router-prefer-named-link-bad) · [正确示例](#ecosystem-vue-router-prefer-named-link-good) | RouterLink 优先使用具名路由对象，而不是静态路径字符串 |
| [`ecosystem/vue-router-prefer-named-push`](#ecosystem-vue-router-prefer-named-push) | [错误示例](#ecosystem-vue-router-prefer-named-push-bad) · [正确示例](#ecosystem-vue-router-prefer-named-push-good) | Vue Router 编程式导航优先使用具名路由对象 |
| [`ecosystem/vue-router-unknown-route`](#ecosystem-vue-router-unknown-route) | [错误示例](#ecosystem-vue-router-unknown-route-bad) · [正确示例](#ecosystem-vue-router-unknown-route-good) | 名称不在完整且已安装的路由器中。 |
| [`ecosystem/vue-test-utils-no-html-snapshot`](#ecosystem-vue-test-utils-no-html-snapshot) | [错误示例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [正确示例](#ecosystem-vue-test-utils-no-html-snapshot-good) | 避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照 |
| [`html/cross-component-nesting`](#html-cross-component-nesting) | [错误示例](#html-cross-component-nesting-bad) · [正确示例](#html-cross-component-nesting-good) | 检查导入组件组合后的实际 HTML 嵌套。 |
| [`html/deprecated-attr`](#html-deprecated-attr) | [错误示例](#html-deprecated-attr-bad) · [正确示例](#html-deprecated-attr-good) | 禁止已弃用的 HTML 属性 |
| [`html/deprecated-element`](#html-deprecated-element) | [错误示例](#html-deprecated-element-bad) · [正确示例](#html-deprecated-element-good) | 禁止已弃用的 HTML 元素 |
| [`html/id-duplication`](#html-id-duplication) | [错误示例](#html-id-duplication-bad) · [正确示例](#html-id-duplication-good) | 禁止重复的元素 ID |
| [`html/no-consecutive-br`](#html-no-consecutive-br) | [错误示例](#html-no-consecutive-br-bad) · [正确示例](#html-no-consecutive-br-good) | 禁止连续的 &lt;br&gt; 元素 |
| [`html/no-dupe-style-properties`](#html-no-dupe-style-properties) | [错误示例](#html-no-dupe-style-properties-bad) · [正确示例](#html-no-dupe-style-properties-good) | 禁止内联 style 属性中重复的属性声明 |
| [`html/no-duplicate-class`](#html-no-duplicate-class) | [错误示例](#html-no-duplicate-class-bad) · [正确示例](#html-no-duplicate-class-good) | 禁止静态 class 属性中重复的类名 |
| [`html/no-duplicate-dt`](#html-no-duplicate-dt) | [错误示例](#html-no-duplicate-dt-bad) · [正确示例](#html-no-duplicate-dt-good) | 禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称 |
| [`html/no-empty-palpable-content`](#html-no-empty-palpable-content) | [错误示例](#html-no-empty-palpable-content-bad) · [正确示例](#html-no-empty-palpable-content-good) | 禁止预期具有可见内容的空元素 |
| [`html/require-datetime`](#html-require-datetime) | [错误示例](#html-require-datetime-bad) · [正确示例](#html-require-datetime-good) | 要求 &lt;time&gt; 元素具有 datetime 属性 |
| [`musea/no-empty-variant`](#musea-no-empty-variant) | [错误示例](#musea-no-empty-variant-bad) · [正确示例](#musea-no-empty-variant-good) | 禁止空的 &lt;variant&gt; 块 |
| [`musea/prefer-design-tokens`](#musea-prefer-design-tokens) | [错误示例](#musea-prefer-design-tokens-bad) · [正确示例](#musea-prefer-design-tokens-good) | 优先使用设计令牌 CSS 变量，而不是硬编码的原始值 |
| [`musea/require-component`](#musea-require-component) | [错误示例](#musea-require-component-bad) · [正确示例](#musea-require-component-good) | 要求 &lt;art&gt; 块具有 component 属性 |
| [`musea/require-title`](#musea-require-title) | [错误示例](#musea-require-title-bad) · [正确示例](#musea-require-title-good) | 要求 &lt;art&gt; 块具有 title 属性 |
| [`musea/unique-variant-names`](#musea-unique-variant-names) | [错误示例](#musea-unique-variant-names-bad) · [正确示例](#musea-unique-variant-names-good) | 要求 variant 名称唯一 |
| [`musea/valid-variant`](#musea-valid-variant) | [错误示例](#musea-valid-variant-bad) · [正确示例](#musea-valid-variant-good) | 要求 &lt;variant&gt; 块具有 name 属性 |
| [`nuxt/no-nuxt-config-test-key`](#nuxt-no-nuxt-config-test-key) | [错误示例](#nuxt-no-nuxt-config-test-key-bad) · [正确示例](#nuxt-no-nuxt-config-test-key-good) | 禁止在 Nuxt 配置中设置 `test` 键 |
| [`nuxt/no-page-meta-runtime-values`](#nuxt-no-page-meta-runtime-values) | [错误示例](#nuxt-no-page-meta-runtime-values-bad) · [正确示例](#nuxt-no-page-meta-runtime-values-good) | 禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行 |
| [`nuxt/nuxt-config-keys-order`](#nuxt-nuxt-config-keys-order) | [错误示例](#nuxt-nuxt-config-keys-order-bad) · [正确示例](#nuxt-nuxt-config-keys-order-good) | 优先使用推荐的 Nuxt 配置属性顺序 |
| [`nuxt/prefer-import-meta`](#nuxt-prefer-import-meta) | [错误示例](#nuxt-prefer-import-meta-bad) · [正确示例](#nuxt-prefer-import-meta-good) | 优先使用 `import.meta.*`，而不是 `process.*` |
| [`petite-vue/no-unsupported-directive`](#petite-vue-no-unsupported-directive) | [错误示例](#petite-vue-no-unsupported-directive-bad) · [正确示例](#petite-vue-no-unsupported-directive-good) | 禁止 petite-vue 不支持的指令 |
| [`petite-vue/valid-v-effect`](#petite-vue-valid-v-effect) | [错误示例](#petite-vue-valid-v-effect-bad) · [正确示例](#petite-vue-valid-v-effect-good) | 要求 v-effect 具有非空表达式 |
| [`petite-vue/valid-v-scope`](#petite-vue-valid-v-scope) | [错误示例](#petite-vue-valid-v-scope-bad) · [正确示例](#petite-vue-valid-v-scope-good) | 要求 v-scope 绑定对象字面量 |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [错误示例](#script-component-options-name-casing-bad) · [正确示例](#script-component-options-name-casing-good) | 要求组件 `name` 选项使用 PascalCase |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [错误示例](#script-custom-event-name-casing-bad) · [正确示例](#script-custom-event-name-casing-good) | 要求发出的自定义事件名称使用 camelCase |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [错误示例](#script-define-emits-declaration-bad) · [正确示例](#script-define-emits-declaration-good) | 要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式 |
| [`script/define-macros-order`](#script-define-macros-order) | [错误示例](#script-define-macros-order-bad) · [正确示例](#script-define-macros-order-good) | 要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序 |
| [`script/define-props-declaration`](#script-define-props-declaration) | [错误示例](#script-define-props-declaration-bad) · [正确示例](#script-define-props-declaration-good) | 要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式 |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [错误示例](#script-define-props-destructuring-bad) · [正确示例](#script-define-props-destructuring-good) | 要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致 |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [错误示例](#script-no-arrow-functions-in-watch-bad) · [正确示例](#script-no-arrow-functions-in-watch-good) | 禁止将箭头函数用作 Options API watch 处理器 |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [错误示例](#script-no-async-in-computed-bad) · [正确示例](#script-no-async-in-computed-good) | 禁止计算属性中的异步函数 |
| [`script/no-boolean-default`](#script-no-boolean-default) | [错误示例](#script-no-boolean-default-bad) · [正确示例](#script-no-boolean-default-good) | 禁止为 Boolean prop 设置默认值 |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [错误示例](#script-no-deep-destructure-in-props-bad) · [正确示例](#script-no-deep-destructure-in-props-good) | 禁止在 defineProps 中进行深层嵌套解构 |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [错误示例](#script-no-deprecated-data-object-declaration-bad) · [正确示例](#script-no-deprecated-data-object-declaration-good) | 禁止将对象字面量作为组件 data 选项（Vue 3 要求函数） |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [错误示例](#script-no-deprecated-destroyed-lifecycle-bad) · [正确示例](#script-no-deprecated-destroyed-lifecycle-good) | 禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子 |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [错误示例](#script-no-deprecated-dollar-listeners-api-bad) · [正确示例](#script-no-deprecated-dollar-listeners-api-good) | 禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs） |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [错误示例](#script-no-deprecated-dollar-scopedslots-api-bad) · [正确示例](#script-no-deprecated-dollar-scopedslots-api-good) | 禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots） |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [错误示例](#script-no-deprecated-events-api-bad) · [正确示例](#script-no-deprecated-events-api-good) | 禁止已移除的 Vue 2 事件 API（$on / $off / $once） |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [错误示例](#script-no-deprecated-props-default-this-bad) · [正确示例](#script-no-deprecated-props-default-this-good) | 禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除） |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [错误示例](#script-no-dupe-keys-bad) · [正确示例](#script-no-dupe-keys-good) | 禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键 |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [错误示例](#script-no-duplicate-attr-inheritance-bad) · [正确示例](#script-no-duplicate-attr-inheritance-good) | 标记重复应用透传属性的组件 |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [错误示例](#script-no-export-in-script-setup-bad) · [正确示例](#script-no-export-in-script-setup-good) | 禁止 &lt;script setup&gt; 内的 export 语句 |
| [`script/no-expose-after-await`](#script-no-expose-after-await) | [错误示例](#script-no-expose-after-await-bad) · [正确示例](#script-no-expose-after-await-good) | 禁止在 await 之后调用 expose() 和 defineExpose() |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null） |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [错误示例](#script-no-import-compiler-macros-bad) · [正确示例](#script-no-import-compiler-macros-good) | 禁止导入自动可用的 Vue 编译器宏 |
| [`script/no-internal-imports`](#script-no-internal-imports) | [错误示例](#script-no-internal-imports-bad) · [正确示例](#script-no-internal-imports-good) | 禁止从 Vue 内部模块导入 |
| [`script/no-lifecycle-after-await`](#script-no-lifecycle-after-await) | [错误示例](#script-no-lifecycle-after-await-bad) · [正确示例](#script-no-lifecycle-after-await-good) | 禁止在 setup() 中 await 之后注册生命周期钩子 |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [错误示例](#script-no-multiple-slot-args-bad) · [正确示例](#script-no-multiple-slot-args-good) | 禁止向作用域插槽函数调用传入多个参数 |
| [`script/no-next-tick`](#script-no-next-tick) | [错误示例](#script-no-next-tick-bad) · [正确示例](#script-no-next-tick-good) | 禁止面向 Vapor 的组件使用 nextTick() |
| [`script/no-options-api`](#script-no-options-api) | [错误示例](#script-no-options-api-bad) · [正确示例](#script-no-options-api-good) | 禁止 Vapor 模式中的 Options API 模式 |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [错误示例](#script-no-potential-component-option-typo-bad) · [正确示例](#script-no-potential-component-option-typo-good) | 标记 Options API 组件选项名称中的疑似拼写错误 |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [错误示例](#script-no-reactive-destructure-bad) · [正确示例](#script-no-reactive-destructure-good) | 禁止导致响应性丢失的响应式对象解构 |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [错误示例](#script-no-ref-as-operand-bad) · [正确示例](#script-no-ref-as-operand-good) | 要求将 ref 绑定变量用作操作数时通过 `.value` 访问 |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [错误示例](#script-no-required-prop-with-default-bad) · [正确示例](#script-no-required-prop-with-default-good) | 禁止 prop 同时具有 required: true 和默认值 |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [错误示例](#script-no-reserved-identifiers-bad) · [正确示例](#script-no-reserved-identifiers-good) | 禁止使用 Vue 编译器保留标识符 |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [错误示例](#script-no-reserved-keys-bad) · [正确示例](#script-no-reserved-keys-good) | 禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键 |
| [`script/no-reserved-props`](#script-no-reserved-props) | [错误示例](#script-no-reserved-props-bad) · [正确示例](#script-no-reserved-props-good) | 禁止在组件 props 声明中使用保留名称 |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [错误示例](#script-no-restricted-globals-bad) · [正确示例](#script-no-restricted-globals-good) | 禁止直接引用必须通过类型化包装器访问的运行时环境全局变量 |
| [`script/no-restricted-members`](#script-no-restricted-members) | [错误示例](#script-no-restricted-members-bad) · [正确示例](#script-no-restricted-members-good) | 禁止项目配置的 object.property 成员访问 |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [错误示例](#script-no-side-effects-in-computed-properties-bad) · [正确示例](#script-no-side-effects-in-computed-properties-good) | 禁止 Options API 计算 getter 中的副作用 |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [错误示例](#script-no-top-level-ref-in-script-bad) · [正确示例](#script-no-top-level-ref-in-script-good) | 禁止顶层 ref/reactive，以防止跨请求状态污染 |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [错误示例](#script-no-unstable-nested-components-bad) · [正确示例](#script-no-unstable-nested-components-good) | 禁止在 setup 或 render 函数内定义组件 |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [错误示例](#script-no-unused-emit-declarations-bad) · [正确示例](#script-no-unused-emit-declarations-good) | 标记已声明却从未发出的事件 |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [错误示例](#script-no-use-computed-property-like-method-bad) · [正确示例](#script-no-use-computed-property-like-method-good) | 禁止像方法一样调用 Options API 计算属性 |
| [`script/no-watch-after-await`](#script-no-watch-after-await) | [错误示例](#script-no-watch-after-await-bad) · [正确示例](#script-no-watch-after-await-good) | 禁止在 setup() 中 await 之后注册未保留停止句柄的侦听器 |
| [`script/no-with-defaults`](#script-no-with-defaults) | [错误示例](#script-no-with-defaults-bad) · [正确示例](#script-no-with-defaults-good) | 不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+） |
| [`script/prefer-computed`](#script-prefer-computed) | [错误示例](#script-prefer-computed-bad) · [正确示例](#script-prefer-computed-good) | 响应式派生状态优先使用 computed() |
| [`script/prefer-define-options`](#script-prefer-define-options) | [错误示例](#script-prefer-define-options-bad) · [正确示例](#script-prefer-define-options-good) | 优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt; |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [错误示例](#script-prefer-import-from-vue-bad) · [正确示例](#script-prefer-import-from-vue-good) | 优先从 'vue' 导入，而不是内部包 |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [错误示例](#script-prefer-ref-over-reactive-bad) · [正确示例](#script-prefer-ref-over-reactive-good) | 建议使用 ref() 而不是 reactive() 管理状态 |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [错误示例](#script-prefer-use-attrs-bad) · [正确示例](#script-prefer-use-attrs-good) | 建议使用 useAttrs() 而不是 context.attrs |
| [`script/prefer-use-id`](#script-prefer-use-id) | [错误示例](#script-prefer-use-id-bad) · [正确示例](#script-prefer-use-id-good) | 建议使用 useId() 生成唯一 ID（Vue 3.5+） |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [错误示例](#script-prefer-use-slots-bad) · [正确示例](#script-prefer-use-slots-good) | 建议使用 useSlots() 而不是 context.slots |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [错误示例](#script-prefer-use-template-ref-bad) · [正确示例](#script-prefer-use-template-ref-good) | 模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+） |
| [`script/require-default-prop`](#script-require-default-prop) | [错误示例](#script-require-default-prop-bad) · [正确示例](#script-require-default-prop-good) | 要求每个可选的非 Boolean prop 具有默认值 |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [错误示例](#script-require-explicit-emits-bad) · [正确示例](#script-require-explicit-emits-good) | 要求发出的事件在 defineEmits 或 emits 选项中声明 |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [错误示例](#script-require-explicit-slots-bad) · [正确示例](#script-require-explicit-slots-good) | 要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型 |
| [`script/require-function-return-type`](#script-require-function-return-type) | [错误示例](#script-require-function-return-type-bad) · [正确示例](#script-require-function-return-type-good) | 要求函数具有返回类型注解 |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [错误示例](#script-require-prop-type-constructor-bad) · [正确示例](#script-require-prop-type-constructor-good) | 要求 prop 的 `type` 值是构造器，而不是字符串字面量 |
| [`script/require-prop-types`](#script-require-prop-types) | [错误示例](#script-require-prop-types-bad) · [正确示例](#script-require-prop-types-good) | 要求每个 prop 声明类型 |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [错误示例](#script-require-symbol-provide-bad) · [正确示例](#script-require-symbol-provide-good) | 建议使用 Symbol 作为 provide/inject 的注入键 |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [错误示例](#script-require-typed-object-prop-bad) · [正确示例](#script-require-typed-object-prop-good) | 要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型 |
| [`script/require-typed-ref`](#script-require-typed-ref) | [错误示例](#script-require-typed-ref-bad) · [正确示例](#script-require-typed-ref-good) | 要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数 |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [错误示例](#script-require-valid-default-prop-bad) · [正确示例](#script-require-valid-default-prop-good) | 要求 prop 默认值符合声明类型 |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [错误示例](#script-return-in-computed-property-bad) · [正确示例](#script-return-in-computed-property-good) | 要求每个计算 getter 返回值 |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [错误示例](#script-return-in-emits-validator-bad) · [正确示例](#script-return-in-emits-validator-good) | 要求每个 Options API emits 校验函数返回值 |
| [`script/valid-define-emits`](#script-valid-define-emits) | [错误示例](#script-valid-define-emits-bad) · [正确示例](#script-valid-define-emits-good) | 要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次） |
| [`script/valid-define-options`](#script-valid-define-options) | [错误示例](#script-valid-define-options-bad) · [正确示例](#script-valid-define-options-good) | 要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots） |
| [`script/valid-define-props`](#script-valid-define-props) | [错误示例](#script-valid-define-props-bad) · [正确示例](#script-valid-define-props-good) | 要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量） |
| [`script/valid-next-tick`](#script-valid-next-tick) | [错误示例](#script-valid-next-tick-bad) · [正确示例](#script-valid-next-tick-good) | 要求 nextTick() 调用结果被等待、链式处理，或提供回调 |
| [`ssr/no-browser-globals-in-ssr`](#ssr-no-browser-globals-in-ssr) | [错误示例](#ssr-no-browser-globals-in-ssr-bad) · [正确示例](#ssr-no-browser-globals-in-ssr-good) | 禁止 SSR 上下文中的浏览器专用全局变量 |
| [`ssr/no-hydration-mismatch`](#ssr-no-hydration-mismatch) | [错误示例](#ssr-no-hydration-mismatch-bad) · [正确示例](#ssr-no-hydration-mismatch-good) | 禁止导致水合不匹配的不确定值 |
| [`type/no-floating-promises`](#type-no-floating-promises) | [错误示例](#type-no-floating-promises-bad) · [正确示例](#type-no-floating-promises-good) | 禁止悬空（未处理）的 Promise |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [错误示例](#type-no-reactivity-loss-bad) · [正确示例](#type-no-reactivity-loss-good) | 禁止赋值和调用中对响应式值取普通快照 |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [错误示例](#type-no-unsafe-template-binding-bad) · [正确示例](#type-no-unsafe-template-binding-good) | 禁止解析为不安全类型的模板绑定 |
| [`type/require-typed-emits`](#type-require-typed-emits) | [错误示例](#type-require-typed-emits-bad) · [正确示例](#type-require-typed-emits-good) | 要求 defineEmits 具有类型定义 |
| [`type/require-typed-props`](#type-require-typed-props) | [错误示例](#type-require-typed-props-bad) · [正确示例](#type-require-typed-props-good) | 要求 defineProps 具有类型定义 |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [错误示例](#type-strict-boolean-expressions-bad) · [正确示例](#type-strict-boolean-expressions-good) | 要求脚本和模板条件使用安全的布尔表达式 |
| [`vapor/no-inline-template`](#vapor-no-inline-template) | [错误示例](#vapor-no-inline-template-bad) · [正确示例](#vapor-no-inline-template-good) | 禁止已弃用的 inline-template 属性 |
| [`vapor/no-vue-lifecycle-events`](#vapor-no-vue-lifecycle-events) | [错误示例](#vapor-no-vue-lifecycle-events-bad) · [正确示例](#vapor-no-vue-lifecycle-events-good) | 禁止 @vue:xxx 元素生命周期事件（Vapor 不支持） |
| [`vapor/prefer-static-class`](#vapor-prefer-static-class) | [错误示例](#vapor-prefer-static-class-bad) · [正确示例](#vapor-prefer-static-class-good) | 字符串字面量优先使用静态 class，而不是动态 class 绑定 |
| [`vapor/require-vapor-attribute`](#vapor-require-vapor-attribute) | [错误示例](#vapor-require-vapor-attribute-bad) · [正确示例](#vapor-require-vapor-attribute-good) | 建议为 script setup 添加 vapor 属性 |
| [`vize:croquis/cf/array-mutation`](#vize-croquis-cf-array-mutation) | [错误示例](#vize-croquis-cf-array-mutation-bad) · [正确示例](#vize-croquis-cf-array-mutation-good) | 通过索引修改数组，无法被该响应式数组跟踪。 |
| [`vize:croquis/cf/async-boundary`](#vize-croquis-cf-async-boundary) | [错误示例](#vize-croquis-cf-async-boundary-bad) · [正确示例](#vize-croquis-cf-async-boundary-good) | 响应式状态跨越异步边界，可能被观察为过期值。 |
| [`vize:croquis/cf/async-no-suspense`](#vize-croquis-cf-async-no-suspense) | [错误示例](#vize-croquis-cf-async-no-suspense-bad) · [正确示例](#vize-croquis-cf-async-no-suspense-good) | 异步组件在没有 Suspense 边界的情况下渲染。 |
| [`vize:croquis/cf/browser-api-ssr`](#vize-croquis-cf-browser-api-ssr) | [错误示例](#vize-croquis-cf-browser-api-ssr-bad) · [正确示例](#vize-croquis-cf-browser-api-ssr-good) | 在组件可能于服务端渲染的位置使用了浏览器专用 API。 |
| [`vize:croquis/cf/circular-dep`](#vize-croquis-cf-circular-dep) | [错误示例](#vize-croquis-cf-circular-dep-bad) · [正确示例](#vize-croquis-cf-circular-dep-good) | 组件相互导入，形成循环。 |
| [`vize:croquis/cf/circular-reactive-dependency`](#vize-croquis-cf-circular-reactive-dependency) | [错误示例](#vize-croquis-cf-circular-reactive-dependency-bad) · [正确示例](#vize-croquis-cf-circular-reactive-dependency-good) | 响应式计算相互依赖，形成循环。 |
| [`vize:croquis/cf/closure-captures-reactive`](#vize-croquis-cf-closure-captures-reactive) | [错误示例](#vize-croquis-cf-closure-captures-reactive-bad) · [正确示例](#vize-croquis-cf-closure-captures-reactive-good) | 闭包捕获了响应式值，无法看到后续更新。 |
| [`vize:croquis/cf/composable-outside-setup`](#vize-croquis-cf-composable-outside-setup) | [错误示例](#vize-croquis-cf-composable-outside-setup-bad) · [正确示例](#vize-croquis-cf-composable-outside-setup-good) | 组合式函数在 `setup` 外被调用。 |
| [`vize:croquis/cf/computed-side-effects`](#vize-croquis-cf-computed-side-effects) | [错误示例](#vize-croquis-cf-computed-side-effects-bad) · [正确示例](#vize-croquis-cf-computed-side-effects-good) | 计算 getter 写入状态或执行其他副作用。 |
| [`vize:croquis/cf/deep-import`](#vize-croquis-cf-deep-import) | [错误示例](#vize-croquis-cf-deep-import-bad) · [正确示例](#vize-croquis-cf-deep-import-good) | 导入链超过项目允许的深度。 |
| [`vize:croquis/cf/destructuring-breaks-reactivity`](#vize-croquis-cf-destructuring-breaks-reactivity) | [错误示例](#vize-croquis-cf-destructuring-breaks-reactivity-bad) · [正确示例](#vize-croquis-cf-destructuring-breaks-reactivity-good) | 解构响应式对象会复制字段并丢失跟踪。 |
| [`vize:croquis/cf/di-outside-setup`](#vize-croquis-cf-di-outside-setup) | [错误示例](#vize-croquis-cf-di-outside-setup-bad) · [正确示例](#vize-croquis-cf-di-outside-setup-good) | `provide` 或 `inject` 在 `setup` 外被调用。 |
| [`vize:croquis/cf/dom-access-without-next-tick`](#vize-croquis-cf-dom-access-without-next-tick) | [错误示例](#vize-croquis-cf-dom-access-without-next-tick-bad) · [正确示例](#vize-croquis-cf-dom-access-without-next-tick-good) | 在 Vue 完成更新刷新之前读取 DOM。 |
| [`vize:croquis/cf/duplicate-id`](#vize-croquis-cf-duplicate-id) | [错误示例](#vize-croquis-cf-duplicate-id-bad) · [正确示例](#vize-croquis-cf-duplicate-id-good) | 多个组件使用同一个元素 id。 |
| [`vize:croquis/cf/event-listener-leak`](#vize-croquis-cf-event-listener-leak) | [错误示例](#vize-croquis-cf-event-listener-leak-bad) · [正确示例](#vize-croquis-cf-event-listener-leak-good) | 注册了事件监听器，却从未移除。 |
| [`vize:croquis/cf/event-modifier`](#vize-croquis-cf-event-modifier) | [错误示例](#vize-croquis-cf-event-modifier-bad) · [正确示例](#vize-croquis-cf-event-modifier-good) | 事件监听器使用了发出事件不支持的修饰符。 |
| [`vize:croquis/cf/hydration-risk`](#vize-croquis-cf-hydration-risk) | [错误示例](#vize-croquis-cf-hydration-risk-bad) · [正确示例](#vize-croquis-cf-hydration-risk-good) | 此诊断代码归类多种响应性问题，包括将 prop 复制到 ref。它并不表示跨文件分析遍会检测所有 Date.now() 表达式。 |
| [`vize:croquis/cf/inherit-attrs-unused`](#vize-croquis-cf-inherit-attrs-unused) | [错误示例](#vize-croquis-cf-inherit-attrs-unused-bad) · [正确示例](#vize-croquis-cf-inherit-attrs-unused-good) | 设置了 `inheritAttrs: false`，但组件从不读取属性。 |
| [`vize:croquis/cf/inject-without-symbol`](#vize-croquis-cf-inject-without-symbol) | [错误示例](#vize-croquis-cf-inject-without-symbol-bad) · [正确示例](#vize-croquis-cf-inject-without-symbol-good) | `inject` 使用普通键，而不是 `InjectionKey` symbol。 |
| [`vize:croquis/cf/injected-async-mutation-race`](#vize-croquis-cf-injected-async-mutation-race) | [错误示例](#vize-croquis-cf-injected-async-mutation-race-bad) · [正确示例](#vize-croquis-cf-injected-async-mutation-race-good) | 注入值被可能发生竞态的异步任务修改。 |
| [`vize:croquis/cf/lifecycle-outside-setup`](#vize-croquis-cf-lifecycle-outside-setup) | [错误示例](#vize-croquis-cf-lifecycle-outside-setup-bad) · [正确示例](#vize-croquis-cf-lifecycle-outside-setup-good) | 生命周期钩子在 `setup` 外注册。 |
| [`vize:croquis/cf/lifecycle-without-cleanup`](#vize-croquis-cf-lifecycle-without-cleanup) | [错误示例](#vize-croquis-cf-lifecycle-without-cleanup-bad) · [正确示例](#vize-croquis-cf-lifecycle-without-cleanup-good) | 生命周期钩子启动任务，却从不清理。 |
| [`vize:croquis/cf/missing-required-prop`](#vize-croquis-cf-missing-required-prop) | [错误示例](#vize-croquis-cf-missing-required-prop-bad) · [正确示例](#vize-croquis-cf-missing-required-prop-good) | 未传入必需的 prop。 |
| [`vize:croquis/cf/missing-suspense`](#vize-croquis-cf-missing-suspense) | [错误示例](#vize-croquis-cf-missing-suspense-bad) · [正确示例](#vize-croquis-cf-missing-suspense-good) | 异步依赖在 Suspense 边界之外使用。 |
| [`vize:croquis/cf/module-scope-reactive`](#vize-croquis-cf-module-scope-reactive) | [错误示例](#vize-croquis-cf-module-scope-reactive-bad) · [正确示例](#vize-croquis-cf-module-scope-reactive-good) | 响应式状态在模块作用域创建，被所有调用者共享。 |
| [`vize:croquis/cf/multi-root-attrs`](#vize-croquis-cf-multi-root-attrs) | [错误示例](#vize-croquis-cf-multi-root-attrs-bad) · [正确示例](#vize-croquis-cf-multi-root-attrs-good) | 多根节点组件接收了属性，却没有放置目标。 |
| [`vize:croquis/cf/mutated-after-escape`](#vize-croquis-cf-mutated-after-escape) | [错误示例](#vize-croquis-cf-mutated-after-escape-bad) · [正确示例](#vize-croquis-cf-mutated-after-escape-good) | 响应式对象逸出其所有者后被修改。 |
| [`vize:croquis/cf/non-reactive-provide`](#vize-croquis-cf-non-reactive-provide) | [错误示例](#vize-croquis-cf-non-reactive-provide-bad) · [正确示例](#vize-croquis-cf-non-reactive-provide-good) | 提供的值不具响应性，后代无法看到更新。 |
| [`vize:croquis/cf/non-unique-id`](#vize-croquis-cf-non-unique-id) | [错误示例](#vize-croquis-cf-non-unique-id-bad) · [正确示例](#vize-croquis-cf-non-unique-id-good) | 循环内的元素 id 对每个列表项并不唯一。 |
| [`vize:croquis/cf/object-identity-comparison`](#vize-croquis-cf-object-identity-comparison) | [错误示例](#vize-croquis-cf-object-identity-comparison-bad) · [正确示例](#vize-croquis-cf-object-identity-comparison-good) | 响应式对象按身份比较，解包后身份会改变。 |
| [`vize:croquis/cf/pinia-getter`](#vize-croquis-cf-pinia-getter) | [错误示例](#vize-croquis-cf-pinia-getter-bad) · [正确示例](#vize-croquis-cf-pinia-getter-good) | Pinia getter 未通过 `storeToRefs` 读取，无法保持响应性。 |
| [`vize:croquis/cf/prop-type-mismatch`](#vize-croquis-cf-prop-type-mismatch) | [错误示例](#vize-croquis-cf-prop-type-mismatch-bad) · [正确示例](#vize-croquis-cf-prop-type-mismatch-good) | 传入的 prop 值与声明类型不匹配。 |
| [`vize:croquis/cf/provide-inject-type`](#vize-croquis-cf-provide-inject-type) | [错误示例](#vize-croquis-cf-provide-inject-type-bad) · [正确示例](#vize-croquis-cf-provide-inject-type-good) | 提供值与其 inject 类型不一致。 |
| [`vize:croquis/cf/provide-without-symbol`](#vize-croquis-cf-provide-without-symbol) | [错误示例](#vize-croquis-cf-provide-without-symbol-bad) · [正确示例](#vize-croquis-cf-provide-without-symbol-good) | `provide` 使用普通键，而不是 `InjectionKey` symbol。 |
| [`vize:croquis/cf/reactive-export`](#vize-croquis-cf-reactive-export) | [错误示例](#vize-croquis-cf-reactive-export-bad) · [正确示例](#vize-croquis-cf-reactive-export-good) | 响应式状态从模块导出。 |
| [`vize:croquis/cf/reactivity-outside-setup`](#vize-croquis-cf-reactivity-outside-setup) | [错误示例](#vize-croquis-cf-reactivity-outside-setup-bad) · [正确示例](#vize-croquis-cf-reactivity-outside-setup-good) | 响应式 API 在 `setup` 外被调用。 |
| [`vize:croquis/cf/reassignment-breaks-reactivity`](#vize-croquis-cf-reassignment-breaks-reactivity) | [错误示例](#vize-croquis-cf-reassignment-breaks-reactivity-bad) · [正确示例](#vize-croquis-cf-reassignment-breaks-reactivity-good) | 重新赋值响应式绑定，将其替换为普通值。 |
| [`vize:croquis/cf/reference-escapes-scope`](#vize-croquis-cf-reference-escapes-scope) | [错误示例](#vize-croquis-cf-reference-escapes-scope-bad) · [正确示例](#vize-croquis-cf-reference-escapes-scope-good) | 响应式引用逸出拥有其生命周期的作用域。 |
| [`vize:croquis/cf/setup-context-violation`](#vize-croquis-cf-setup-context-violation) | [错误示例](#vize-croquis-cf-setup-context-violation-bad) · [正确示例](#vize-croquis-cf-setup-context-violation-good) | setup 上下文被以 Vue 不允许的方式使用。 |
| [`vize:croquis/cf/shallow-deep-access`](#vize-croquis-cf-shallow-deep-access) | [错误示例](#vize-croquis-cf-shallow-deep-access-bad) · [正确示例](#vize-croquis-cf-shallow-deep-access-good) | 读取 `shallowReactive` 或 `shallowRef` 的深层属性，却假定它受到跟踪。 |
| [`vize:croquis/cf/spread-breaks-reactivity`](#vize-croquis-cf-spread-breaks-reactivity) | [错误示例](#vize-croquis-cf-spread-breaks-reactivity-bad) · [正确示例](#vize-croquis-cf-spread-breaks-reactivity-good) | 展开响应式对象会复制值并丢失跟踪。 |
| [`vize:croquis/cf/suspense-no-fallback`](#vize-croquis-cf-suspense-no-fallback) | [错误示例](#vize-croquis-cf-suspense-no-fallback-bad) · [正确示例](#vize-croquis-cf-suspense-no-fallback-good) | `<Suspense>` 没有后备内容。 |
| [`vize:croquis/cf/template-ref-timing`](#vize-croquis-cf-template-ref-timing) | [错误示例](#vize-croquis-cf-template-ref-timing-bad) · [正确示例](#vize-croquis-cf-template-ref-timing-good) | 模板 ref 在组件挂载之前被读取。 |
| [`vize:croquis/cf/toraw-mutation`](#vize-croquis-cf-toraw-mutation) | [错误示例](#vize-croquis-cf-toraw-mutation-bad) · [正确示例](#vize-croquis-cf-toraw-mutation-good) | 使用 `toRaw` 后修改了原始对象。 |
| [`vize:croquis/cf/uncaught-error`](#vize-croquis-cf-uncaught-error) | [错误示例](#vize-croquis-cf-uncaught-error-bad) · [正确示例](#vize-croquis-cf-uncaught-error-good) | 组件可能抛错，却没有错误边界捕获。 |
| [`vize:croquis/cf/undeclared-emit`](#vize-croquis-cf-undeclared-emit) | [错误示例](#vize-croquis-cf-undeclared-emit-bad) · [正确示例](#vize-croquis-cf-undeclared-emit-good) | 组件发出了未声明的事件。 |
| [`vize:croquis/cf/undeclared-prop`](#vize-croquis-cf-undeclared-prop) | [错误示例](#vize-croquis-cf-undeclared-prop-bad) · [正确示例](#vize-croquis-cf-undeclared-prop-good) | 父组件传入了子组件未声明的 prop。 |
| [`vize:croquis/cf/undefined-slot`](#vize-croquis-cf-undefined-slot) | [错误示例](#vize-croquis-cf-undefined-slot-bad) · [正确示例](#vize-croquis-cf-undefined-slot-good) | 父组件填充了子组件未暴露的插槽。 |
| [`vize:croquis/cf/unhandled-event`](#vize-croquis-cf-unhandled-event) | [错误示例](#vize-croquis-cf-unhandled-event-bad) · [正确示例](#vize-croquis-cf-unhandled-event-good) | 子组件发出的事件没有父组件处理。 |
| [`vize:croquis/cf/unmatched-inject`](#vize-croquis-cf-unmatched-inject) | [错误示例](#vize-croquis-cf-unmatched-inject-bad) · [正确示例](#vize-croquis-cf-unmatched-inject-good) | `inject` 指定的键没有任何祖先提供。 |
| [`vize:croquis/cf/unmatched-listener`](#vize-croquis-cf-unmatched-listener) | [错误示例](#vize-croquis-cf-unmatched-listener-bad) · [正确示例](#vize-croquis-cf-unmatched-listener-good) | 父组件监听了子组件不发出的事件。 |
| [`vize:croquis/cf/unregistered-component`](#vize-croquis-cf-unregistered-component) | [错误示例](#vize-croquis-cf-unregistered-component-bad) · [正确示例](#vize-croquis-cf-unregistered-component-good) | 模板使用了未注册或导入的组件。 |
| [`vize:croquis/cf/unresolved-import`](#vize-croquis-cf-unresolved-import) | [错误示例](#vize-croquis-cf-unresolved-import-bad) · [正确示例](#vize-croquis-cf-unresolved-import-good) | 导入无法解析为模块。 |
| [`vize:croquis/cf/unused-attrs`](#vize-croquis-cf-unused-attrs) | [错误示例](#vize-croquis-cf-unused-attrs-bad) · [正确示例](#vize-croquis-cf-unused-attrs-good) | 透传属性传给了未使用它们的多根节点组件。 |
| [`vize:croquis/cf/unused-emit`](#vize-croquis-cf-unused-emit) | [错误示例](#vize-croquis-cf-unused-emit-bad) · [正确示例](#vize-croquis-cf-unused-emit-good) | 已声明的 emit 从未使用。 |
| [`vize:croquis/cf/unused-provide`](#vize-croquis-cf-unused-provide) | [错误示例](#vize-croquis-cf-unused-provide-bad) · [正确示例](#vize-croquis-cf-unused-provide-good) | 提供的键从未被注入。 |
| [`vize:croquis/cf/value-extraction-breaks-reactivity`](#vize-croquis-cf-value-extraction-breaks-reactivity) | [错误示例](#vize-croquis-cf-value-extraction-breaks-reactivity-bad) · [正确示例](#vize-croquis-cf-value-extraction-breaks-reactivity-good) | 将响应式值读取到局部变量后丢失后续更新。 |
| [`vize:croquis/cf/watch-can-be-computed`](#vize-croquis-cf-watch-can-be-computed) | [错误示例](#vize-croquis-cf-watch-can-be-computed-bad) · [正确示例](#vize-croquis-cf-watch-can-be-computed-good) | 侦听器只将值复制到状态，可以改为计算属性。 |
| [`vize:croquis/cf/watcheffect-async`](#vize-croquis-cf-watcheffect-async) | [错误示例](#vize-croquis-cf-watcheffect-async-bad) · [正确示例](#vize-croquis-cf-watcheffect-async-good) | `watchEffect` 启动异步任务，却无法清理上一轮执行。 |
| [`vize:croquis/cf/watcher-outside-setup`](#vize-croquis-cf-watcher-outside-setup) | [错误示例](#vize-croquis-cf-watcher-outside-setup-bad) · [正确示例](#vize-croquis-cf-watcher-outside-setup-good) | `watch` 或 `watchEffect` 在 `setup` 外被调用。 |
| [`vue/a11y-img-alt`](#vue-a11y-img-alt) | [错误示例](#vue-a11y-img-alt-bad) · [正确示例](#vue-a11y-img-alt-good) | 要求图片提供 alt 属性，以支持无障碍访问 |
| [`vue/attribute-hyphenation`](#vue-attribute-hyphenation) | [错误示例](#vue-attribute-hyphenation-bad) · [正确示例](#vue-attribute-hyphenation-good) | 规范自定义组件的属性命名风格 |
| [`vue/attribute-order`](#vue-attribute-order) | [错误示例](#vue-attribute-order-bad) · [正确示例](#vue-attribute-order-good) | 要求属性保持一致的排列顺序 |
| [`vue/component-definition-name-casing`](#vue-component-definition-name-casing) | [错误示例](#vue-component-definition-name-casing-bad) · [正确示例](#vue-component-definition-name-casing-good) | 要求组件定义名称使用 PascalCase 或 kebab-case |
| [`vue/component-name-in-template-casing`](#vue-component-name-in-template-casing) | [错误示例](#vue-component-name-in-template-casing-bad) · [正确示例](#vue-component-name-in-template-casing-good) | 要求模板中的组件名称使用指定的大小写风格 |
| [`vue/cross-file-attrs-fallthrough`](#vue-cross-file-attrs-fallthrough) | [错误示例](#vue-cross-file-attrs-fallthrough-bad) · [正确示例](#vue-cross-file-attrs-fallthrough-good) | 父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。 |
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
| [`vue/no-deprecated-v-is`](#vue-no-deprecated-v-is) | [错误示例](#vue-no-deprecated-v-is-bad) · [正确示例](#vue-no-deprecated-v-is-good) | 禁止使用已废弃的 `v-is` 指令 |
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
| [`vue/valid-v-else-if`](#vue-valid-v-else-if) | [错误示例](#vue-valid-v-else-if-bad) · [正确示例](#vue-valid-v-else-if-good) | 要求 `v-else-if` 指令有效 |
| [`vue/valid-v-for`](#vue-valid-v-for) | [错误示例](#vue-valid-v-for-bad) · [正确示例](#vue-valid-v-for-good) | 要求 `v-for` 指令有效 |
| [`vue/valid-v-html`](#vue-valid-v-html) | [错误示例](#vue-valid-v-html-bad) · [正确示例](#vue-valid-v-html-good) | 要求 `v-html` 指令有效 |
| [`vue/valid-v-if`](#vue-valid-v-if) | [错误示例](#vue-valid-v-if-bad) · [正确示例](#vue-valid-v-if-good) | 要求 `v-if` 指令有效 |
| [`vue/valid-v-is`](#vue-valid-v-is) | [错误示例](#vue-valid-v-is-bad) · [正确示例](#vue-valid-v-is-good) | 要求 `v-is` 指令有效 |
| [`vue/valid-v-memo`](#vue-valid-v-memo) | [错误示例](#vue-valid-v-memo-bad) · [正确示例](#vue-valid-v-memo-good) | 要求 `v-memo` 指令有效 |
| [`vue/valid-v-model`](#vue-valid-v-model) | [错误示例](#vue-valid-v-model-bad) · [正确示例](#vue-valid-v-model-good) | 要求 `v-model` 指令有效 |
| [`vue/valid-v-on`](#vue-valid-v-on) | [错误示例](#vue-valid-v-on-bad) · [正确示例](#vue-valid-v-on-good) | 要求 `v-on` 指令有效 |
| [`vue/valid-v-once`](#vue-valid-v-once) | [错误示例](#vue-valid-v-once-bad) · [正确示例](#vue-valid-v-once-good) | 要求 `v-once` 指令有效 |
| [`vue/valid-v-pre`](#vue-valid-v-pre) | [错误示例](#vue-valid-v-pre-bad) · [正确示例](#vue-valid-v-pre-good) | 要求 `v-pre` 指令有效 |
| [`vue/valid-v-show`](#vue-valid-v-show) | [错误示例](#vue-valid-v-show-bad) · [正确示例](#vue-valid-v-show-good) | 要求 `v-show` 指令有效 |
| [`vue/valid-v-slot`](#vue-valid-v-slot) | [错误示例](#vue-valid-v-slot-bad) · [正确示例](#vue-valid-v-slot-good) | 要求 `v-slot` 指令有效 |
| [`vue/valid-v-text`](#vue-valid-v-text) | [错误示例](#vue-valid-v-text-bad) · [正确示例](#vue-valid-v-text-good) | 要求 `v-text` 指令有效 |
| [`vue/warn-custom-block`](#vue-warn-custom-block) | [错误示例](#vue-warn-custom-block-bad) · [正确示例](#vue-warn-custom-block-good) | 警告 SFC 文件中的自定义块 |
| [`vue/warn-custom-directive`](#vue-warn-custom-directive) | [错误示例](#vue-warn-custom-directive-bad) · [正确示例](#vue-warn-custom-directive-good) | 警告需要注册的自定义指令 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `a11y/alt-text`

要求媒体元素提供替代文本

<span id="a11y-alt-text-bad"></span>

[错误示例](./accessibility.md#a11y-alt-text-bad)

<span id="a11y-alt-text-good"></span>

[正确示例](./accessibility.md#a11y-alt-text-good)

[分组示例](./accessibility.md#a11y-alt-text) · [规则参考](/rules/reference/a11y-alt-text.md)

### `a11y/anchor-has-content`

要求锚点元素具有可访问的内容

<span id="a11y-anchor-has-content-bad"></span>

[错误示例](./accessibility.md#a11y-anchor-has-content-bad)

<span id="a11y-anchor-has-content-good"></span>

[正确示例](./accessibility.md#a11y-anchor-has-content-good)

[分组示例](./accessibility.md#a11y-anchor-has-content) · [规则参考](/rules/reference/a11y-anchor-has-content.md)

### `a11y/anchor-is-valid`

要求锚点元素具有有效的 href

<span id="a11y-anchor-is-valid-bad"></span>

[错误示例](./accessibility.md#a11y-anchor-is-valid-bad)

<span id="a11y-anchor-is-valid-good"></span>

[正确示例](./accessibility.md#a11y-anchor-is-valid-good)

[分组示例](./accessibility.md#a11y-anchor-is-valid) · [规则参考](/rules/reference/a11y-anchor-is-valid.md)

### `a11y/aria-props`

禁止无效的 ARIA 属性

<span id="a11y-aria-props-bad"></span>

[错误示例](./accessibility.md#a11y-aria-props-bad)

<span id="a11y-aria-props-good"></span>

[正确示例](./accessibility.md#a11y-aria-props-good)

[分组示例](./accessibility.md#a11y-aria-props) · [规则参考](/rules/reference/a11y-aria-props.md)

### `a11y/aria-role`

具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色

<span id="a11y-aria-role-bad"></span>

[错误示例](./accessibility.md#a11y-aria-role-bad)

<span id="a11y-aria-role-good"></span>

[正确示例](./accessibility.md#a11y-aria-role-good)

[分组示例](./accessibility.md#a11y-aria-role) · [规则参考](/rules/reference/a11y-aria-role.md)

### `a11y/aria-unsupported-elements`

禁止在不支持 ARIA 的元素上使用 ARIA 属性

<span id="a11y-aria-unsupported-elements-bad"></span>

[错误示例](./accessibility.md#a11y-aria-unsupported-elements-bad)

<span id="a11y-aria-unsupported-elements-good"></span>

[正确示例](./accessibility.md#a11y-aria-unsupported-elements-good)

[分组示例](./accessibility.md#a11y-aria-unsupported-elements) · [规则参考](/rules/reference/a11y-aria-unsupported-elements.md)

### `a11y/click-events-have-key-events`

要求点击事件配有键盘事件处理器

<span id="a11y-click-events-have-key-events-bad"></span>

[错误示例](./accessibility.md#a11y-click-events-have-key-events-bad)

<span id="a11y-click-events-have-key-events-good"></span>

[正确示例](./accessibility.md#a11y-click-events-have-key-events-good)

[分组示例](./accessibility.md#a11y-click-events-have-key-events) · [规则参考](/rules/reference/a11y-click-events-have-key-events.md)

### `a11y/form-control-has-label`

要求表单控件具有相关联的标签

<span id="a11y-form-control-has-label-bad"></span>

[错误示例](./accessibility.md#a11y-form-control-has-label-bad)

<span id="a11y-form-control-has-label-good"></span>

[正确示例](./accessibility.md#a11y-form-control-has-label-good)

[分组示例](./accessibility.md#a11y-form-control-has-label) · [规则参考](/rules/reference/a11y-form-control-has-label.md)

### `a11y/heading-has-content`

要求标题元素具有可访问的内容

<span id="a11y-heading-has-content-bad"></span>

[错误示例](./accessibility.md#a11y-heading-has-content-bad)

<span id="a11y-heading-has-content-good"></span>

[正确示例](./accessibility.md#a11y-heading-has-content-good)

[分组示例](./accessibility.md#a11y-heading-has-content) · [规则参考](/rules/reference/a11y-heading-has-content.md)

### `a11y/heading-levels`

禁止跳过标题层级

<span id="a11y-heading-levels-bad"></span>

[错误示例](./accessibility.md#a11y-heading-levels-bad)

<span id="a11y-heading-levels-good"></span>

[正确示例](./accessibility.md#a11y-heading-levels-good)

[分组示例](./accessibility.md#a11y-heading-levels) · [规则参考](/rules/reference/a11y-heading-levels.md)

### `a11y/iframe-has-title`

要求 iframe 元素具有 title 属性

<span id="a11y-iframe-has-title-bad"></span>

[错误示例](./accessibility.md#a11y-iframe-has-title-bad)

<span id="a11y-iframe-has-title-good"></span>

[正确示例](./accessibility.md#a11y-iframe-has-title-good)

[分组示例](./accessibility.md#a11y-iframe-has-title) · [规则参考](/rules/reference/a11y-iframe-has-title.md)

### `a11y/img-alt`

要求图片提供 alt 属性，以支持无障碍访问

<span id="a11y-img-alt-bad"></span>

[错误示例](./accessibility.md#a11y-img-alt-bad)

<span id="a11y-img-alt-good"></span>

[正确示例](./accessibility.md#a11y-img-alt-good)

[分组示例](./accessibility.md#a11y-img-alt) · [规则参考](/rules/reference/a11y-img-alt.md)

### `a11y/interactive-supports-focus`

要求具有交互角色的元素可获得焦点

<span id="a11y-interactive-supports-focus-bad"></span>

[错误示例](./accessibility.md#a11y-interactive-supports-focus-bad)

<span id="a11y-interactive-supports-focus-good"></span>

[正确示例](./accessibility.md#a11y-interactive-supports-focus-good)

[分组示例](./accessibility.md#a11y-interactive-supports-focus) · [规则参考](/rules/reference/a11y-interactive-supports-focus.md)

### `a11y/label-has-for`

要求标签具有相关联的表单控件

<span id="a11y-label-has-for-bad"></span>

[错误示例](./accessibility.md#a11y-label-has-for-bad)

<span id="a11y-label-has-for-good"></span>

[正确示例](./accessibility.md#a11y-label-has-for-good)

[分组示例](./accessibility.md#a11y-label-has-for) · [规则参考](/rules/reference/a11y-label-has-for.md)

### `a11y/landmark-roles`

验证地标角色的位置及唯一性

<span id="a11y-landmark-roles-bad"></span>

[错误示例](./accessibility.md#a11y-landmark-roles-bad)

<span id="a11y-landmark-roles-good"></span>

[正确示例](./accessibility.md#a11y-landmark-roles-good)

[分组示例](./accessibility.md#a11y-landmark-roles) · [规则参考](/rules/reference/a11y-landmark-roles.md)

### `a11y/media-has-caption`

要求媒体元素具有字幕

<span id="a11y-media-has-caption-bad"></span>

[错误示例](./accessibility.md#a11y-media-has-caption-bad)

<span id="a11y-media-has-caption-good"></span>

[正确示例](./accessibility.md#a11y-media-has-caption-good)

[分组示例](./accessibility.md#a11y-media-has-caption) · [规则参考](/rules/reference/a11y-media-has-caption.md)

### `a11y/mouse-events-have-key-events`

要求鼠标事件配有 focus/blur 事件

<span id="a11y-mouse-events-have-key-events-bad"></span>

[错误示例](./accessibility.md#a11y-mouse-events-have-key-events-bad)

<span id="a11y-mouse-events-have-key-events-good"></span>

[正确示例](./accessibility.md#a11y-mouse-events-have-key-events-good)

[分组示例](./accessibility.md#a11y-mouse-events-have-key-events) · [规则参考](/rules/reference/a11y-mouse-events-have-key-events.md)

### `a11y/no-access-key`

禁止使用 accesskey 属性

<span id="a11y-no-access-key-bad"></span>

[错误示例](./accessibility.md#a11y-no-access-key-bad)

<span id="a11y-no-access-key-good"></span>

[正确示例](./accessibility.md#a11y-no-access-key-good)

[分组示例](./accessibility.md#a11y-no-access-key) · [规则参考](/rules/reference/a11y-no-access-key.md)

### `a11y/no-aria-hidden-on-focusable`

禁止在可获得焦点的元素上使用 aria-hidden="true"

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

[错误示例](./accessibility.md#a11y-no-aria-hidden-on-focusable-bad)

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

[正确示例](./accessibility.md#a11y-no-aria-hidden-on-focusable-good)

[分组示例](./accessibility.md#a11y-no-aria-hidden-on-focusable) · [规则参考](/rules/reference/a11y-no-aria-hidden-on-focusable.md)

### `a11y/no-autofocus`

禁止使用 autofocus 属性

<span id="a11y-no-autofocus-bad"></span>

[错误示例](./accessibility.md#a11y-no-autofocus-bad)

<span id="a11y-no-autofocus-good"></span>

[正确示例](./accessibility.md#a11y-no-autofocus-good)

[分组示例](./accessibility.md#a11y-no-autofocus) · [规则参考](/rules/reference/a11y-no-autofocus.md)

### `a11y/no-distracting-elements`

禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素

<span id="a11y-no-distracting-elements-bad"></span>

[错误示例](./accessibility.md#a11y-no-distracting-elements-bad)

<span id="a11y-no-distracting-elements-good"></span>

[正确示例](./accessibility.md#a11y-no-distracting-elements-good)

[分组示例](./accessibility.md#a11y-no-distracting-elements) · [规则参考](/rules/reference/a11y-no-distracting-elements.md)

### `a11y/no-i-for-icon`

禁止使用 &lt;i&gt; 元素表示图标

<span id="a11y-no-i-for-icon-bad"></span>

[错误示例](./accessibility.md#a11y-no-i-for-icon-bad)

<span id="a11y-no-i-for-icon-good"></span>

[正确示例](./accessibility.md#a11y-no-i-for-icon-good)

[分组示例](./accessibility.md#a11y-no-i-for-icon) · [规则参考](/rules/reference/a11y-no-i-for-icon.md)

### `a11y/no-redundant-roles`

禁止多余的 ARIA 角色

<span id="a11y-no-redundant-roles-bad"></span>

[错误示例](./accessibility.md#a11y-no-redundant-roles-bad)

<span id="a11y-no-redundant-roles-good"></span>

[正确示例](./accessibility.md#a11y-no-redundant-roles-good)

[分组示例](./accessibility.md#a11y-no-redundant-roles) · [规则参考](/rules/reference/a11y-no-redundant-roles.md)

### `a11y/no-refer-to-non-existent-id`

禁止引用不存在的 ID

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

[错误示例](./accessibility.md#a11y-no-refer-to-non-existent-id-bad)

<span id="a11y-no-refer-to-non-existent-id-good"></span>

[正确示例](./accessibility.md#a11y-no-refer-to-non-existent-id-good)

[分组示例](./accessibility.md#a11y-no-refer-to-non-existent-id) · [规则参考](/rules/reference/a11y-no-refer-to-non-existent-id.md)

### `a11y/no-role-presentation-on-focusable`

禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

[错误示例](./accessibility.md#a11y-no-role-presentation-on-focusable-bad)

<span id="a11y-no-role-presentation-on-focusable-good"></span>

[正确示例](./accessibility.md#a11y-no-role-presentation-on-focusable-good)

[分组示例](./accessibility.md#a11y-no-role-presentation-on-focusable) · [规则参考](/rules/reference/a11y-no-role-presentation-on-focusable.md)

### `a11y/no-static-element-interactions`

禁止在静态元素上设置事件处理器

<span id="a11y-no-static-element-interactions-bad"></span>

[错误示例](./accessibility.md#a11y-no-static-element-interactions-bad)

<span id="a11y-no-static-element-interactions-good"></span>

[正确示例](./accessibility.md#a11y-no-static-element-interactions-good)

[分组示例](./accessibility.md#a11y-no-static-element-interactions) · [规则参考](/rules/reference/a11y-no-static-element-interactions.md)

### `a11y/placeholder-label-option`

要求 select 的占位选项具有 disabled 或 hidden

<span id="a11y-placeholder-label-option-bad"></span>

[错误示例](./accessibility.md#a11y-placeholder-label-option-bad)

<span id="a11y-placeholder-label-option-good"></span>

[正确示例](./accessibility.md#a11y-placeholder-label-option-good)

[分组示例](./accessibility.md#a11y-placeholder-label-option) · [规则参考](/rules/reference/a11y-placeholder-label-option.md)

### `a11y/role-has-required-aria-props`

要求 ARIA 角色具有必需属性

<span id="a11y-role-has-required-aria-props-bad"></span>

[错误示例](./accessibility.md#a11y-role-has-required-aria-props-bad)

<span id="a11y-role-has-required-aria-props-good"></span>

[正确示例](./accessibility.md#a11y-role-has-required-aria-props-good)

[分组示例](./accessibility.md#a11y-role-has-required-aria-props) · [规则参考](/rules/reference/a11y-role-has-required-aria-props.md)

### `a11y/tabindex-no-positive`

禁止正数 tabindex 值

<span id="a11y-tabindex-no-positive-bad"></span>

[错误示例](./accessibility.md#a11y-tabindex-no-positive-bad)

<span id="a11y-tabindex-no-positive-good"></span>

[正确示例](./accessibility.md#a11y-tabindex-no-positive-good)

[分组示例](./accessibility.md#a11y-tabindex-no-positive) · [规则参考](/rules/reference/a11y-tabindex-no-positive.md)

### `a11y/use-list`

建议为类似项目符号的文本使用列表元素

<span id="a11y-use-list-bad"></span>

[错误示例](./accessibility.md#a11y-use-list-bad)

<span id="a11y-use-list-good"></span>

[正确示例](./accessibility.md#a11y-use-list-good)

[分组示例](./accessibility.md#a11y-use-list) · [规则参考](/rules/reference/a11y-use-list.md)

### `css/no-display-none`

建议使用 v-show 代替 display: none

<span id="css-no-display-none-bad"></span>

[错误示例](./musea-and-css.md#css-no-display-none-bad)

<span id="css-no-display-none-good"></span>

[正确示例](./musea-and-css.md#css-no-display-none-good)

[分组示例](./musea-and-css.md#css-no-display-none) · [规则参考](/rules/reference/css-no-display-none.md)

### `css/no-hardcoded-values`

建议使用 CSS 变量代替硬编码值

<span id="css-no-hardcoded-values-bad"></span>

[错误示例](./musea-and-css.md#css-no-hardcoded-values-bad)

<span id="css-no-hardcoded-values-good"></span>

[正确示例](./musea-and-css.md#css-no-hardcoded-values-good)

[分组示例](./musea-and-css.md#css-no-hardcoded-values) · [规则参考](/rules/reference/css-no-hardcoded-values.md)

### `css/no-id-selectors`

不建议在 CSS 中使用 ID 选择器

<span id="css-no-id-selectors-bad"></span>

[错误示例](./musea-and-css.md#css-no-id-selectors-bad)

<span id="css-no-id-selectors-good"></span>

[正确示例](./musea-and-css.md#css-no-id-selectors-good)

[分组示例](./musea-and-css.md#css-no-id-selectors) · [规则参考](/rules/reference/css-no-id-selectors.md)

### `css/no-important`

不建议在 CSS 中使用 !important

<span id="css-no-important-bad"></span>

[错误示例](./musea-and-css.md#css-no-important-bad)

<span id="css-no-important-good"></span>

[正确示例](./musea-and-css.md#css-no-important-good)

[分组示例](./musea-and-css.md#css-no-important) · [规则参考](/rules/reference/css-no-important.md)

### `css/no-utility-classes`

警告在组件样式中实现工具类

<span id="css-no-utility-classes-bad"></span>

[错误示例](./musea-and-css.md#css-no-utility-classes-bad)

<span id="css-no-utility-classes-good"></span>

[正确示例](./musea-and-css.md#css-no-utility-classes-good)

[分组示例](./musea-and-css.md#css-no-utility-classes) · [规则参考](/rules/reference/css-no-utility-classes.md)

### `css/no-v-bind-performance`

警告 CSS v-bind() 的性能开销

<span id="css-no-v-bind-performance-bad"></span>

[错误示例](./musea-and-css.md#css-no-v-bind-performance-bad)

<span id="css-no-v-bind-performance-good"></span>

[正确示例](./musea-and-css.md#css-no-v-bind-performance-good)

[分组示例](./musea-and-css.md#css-no-v-bind-performance) · [规则参考](/rules/reference/css-no-v-bind-performance.md)

### `css/prefer-logical-properties`

建议使用 CSS 逻辑属性，以更好地支持国际化

<span id="css-prefer-logical-properties-bad"></span>

[错误示例](./musea-and-css.md#css-prefer-logical-properties-bad)

<span id="css-prefer-logical-properties-good"></span>

[正确示例](./musea-and-css.md#css-prefer-logical-properties-good)

[分组示例](./musea-and-css.md#css-prefer-logical-properties) · [规则参考](/rules/reference/css-prefer-logical-properties.md)

### `css/prefer-nested-selectors`

建议为后代选择器使用 CSS 嵌套

<span id="css-prefer-nested-selectors-bad"></span>

[错误示例](./musea-and-css.md#css-prefer-nested-selectors-bad)

<span id="css-prefer-nested-selectors-good"></span>

[正确示例](./musea-and-css.md#css-prefer-nested-selectors-good)

[分组示例](./musea-and-css.md#css-prefer-nested-selectors) · [规则参考](/rules/reference/css-prefer-nested-selectors.md)

### `css/prefer-slotted`

建议使用 ::v-slotted() 为插槽内容设置样式

<span id="css-prefer-slotted-bad"></span>

[错误示例](./musea-and-css.md#css-prefer-slotted-bad)

<span id="css-prefer-slotted-good"></span>

[正确示例](./musea-and-css.md#css-prefer-slotted-good)

[分组示例](./musea-and-css.md#css-prefer-slotted) · [规则参考](/rules/reference/css-prefer-slotted.md)

### `css/require-font-display`

要求 @font-face 规则具有 font-display

<span id="css-require-font-display-bad"></span>

[错误示例](./musea-and-css.md#css-require-font-display-bad)

<span id="css-require-font-display-good"></span>

[正确示例](./musea-and-css.md#css-require-font-display-good)

[分组示例](./musea-and-css.md#css-require-font-display) · [规则参考](/rules/reference/css-require-font-display.md)

### `ecosystem/nuxt-prefer-nuxt-link`

应用内部链接优先使用 NuxtLink

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

[错误示例](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-bad)

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

[正确示例](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link-good)

[分组示例](./ecosystem.md#ecosystem-nuxt-prefer-nuxt-link) · [规则参考](/rules/reference/ecosystem-nuxt-prefer-nuxt-link.md)

### `ecosystem/pinia-prefer-store-to-refs`

解构 Pinia store 时优先使用 storeToRefs()

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

[错误示例](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-bad)

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

[正确示例](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs-good)

[分组示例](./ecosystem.md#ecosystem-pinia-prefer-store-to-refs) · [规则参考](/rules/reference/ecosystem-pinia-prefer-store-to-refs.md)

### `ecosystem/router-link-require-to`

要求 RouterLink 和 NuxtLink 组件具有 `to` 目标

<span id="ecosystem-router-link-require-to-bad"></span>

[错误示例](./ecosystem.md#ecosystem-router-link-require-to-bad)

<span id="ecosystem-router-link-require-to-good"></span>

[正确示例](./ecosystem.md#ecosystem-router-link-require-to-good)

[分组示例](./ecosystem.md#ecosystem-router-link-require-to) · [规则参考](/rules/reference/ecosystem-router-link-require-to.md)

### `ecosystem/void-link-require-href`

要求 Void Vue Link 组件具有 `href`

<span id="ecosystem-void-link-require-href-bad"></span>

[错误示例](./ecosystem.md#ecosystem-void-link-require-href-bad)

<span id="ecosystem-void-link-require-href-good"></span>

[正确示例](./ecosystem.md#ecosystem-void-link-require-href-good)

[分组示例](./ecosystem.md#ecosystem-void-link-require-href) · [规则参考](/rules/reference/ecosystem-void-link-require-href.md)

### `ecosystem/void-link-valid-method`

验证 Void Vue Link 的静态 method prop

<span id="ecosystem-void-link-valid-method-bad"></span>

[错误示例](./ecosystem.md#ecosystem-void-link-valid-method-bad)

<span id="ecosystem-void-link-valid-method-good"></span>

[正确示例](./ecosystem.md#ecosystem-void-link-valid-method-good)

[分组示例](./ecosystem.md#ecosystem-void-link-valid-method) · [规则参考](/rules/reference/ecosystem-void-link-valid-method.md)

### `ecosystem/vue-i18n-no-missing-key`

报告本地 SFC 消息中不存在的静态 vue-i18n 键

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

[错误示例](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-bad)

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

[正确示例](./ecosystem.md#ecosystem-vue-i18n-no-missing-key-good)

[分组示例](./ecosystem.md#ecosystem-vue-i18n-no-missing-key) · [规则参考](/rules/reference/ecosystem-vue-i18n-no-missing-key.md)

### `ecosystem/vue-router-extra-param`

路由未声明 tab；Vue Router 会丢弃它。

<span id="ecosystem-vue-router-extra-param-bad"></span>

[错误示例](./cross-file.md#ecosystem-vue-router-extra-param-bad)

<span id="ecosystem-vue-router-extra-param-good"></span>

[正确示例](./cross-file.md#ecosystem-vue-router-extra-param-good)

[分组示例](./cross-file.md#ecosystem-vue-router-extra-param) · [规则参考](/rules/project/ecosystem-vue-router-extra-param.md)

### `ecosystem/vue-router-missing-param`

缺少必需的 postId；依赖当前路由并不稳健。

<span id="ecosystem-vue-router-missing-param-bad"></span>

[错误示例](./cross-file.md#ecosystem-vue-router-missing-param-bad)

<span id="ecosystem-vue-router-missing-param-good"></span>

[正确示例](./cross-file.md#ecosystem-vue-router-missing-param-good)

[分组示例](./cross-file.md#ecosystem-vue-router-missing-param) · [规则参考](/rules/project/ecosystem-vue-router-missing-param.md)

### `ecosystem/vue-router-param-type`

postId 不是可重复参数，因此数组无效。

<span id="ecosystem-vue-router-param-type-bad"></span>

[错误示例](./cross-file.md#ecosystem-vue-router-param-type-bad)

<span id="ecosystem-vue-router-param-type-good"></span>

[正确示例](./cross-file.md#ecosystem-vue-router-param-type-good)

[分组示例](./cross-file.md#ecosystem-vue-router-param-type) · [规则参考](/rules/project/ecosystem-vue-router-param-type.md)

### `ecosystem/vue-router-prefer-named-link`

RouterLink 优先使用具名路由对象，而不是静态路径字符串

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

[错误示例](./ecosystem.md#ecosystem-vue-router-prefer-named-link-bad)

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

[正确示例](./ecosystem.md#ecosystem-vue-router-prefer-named-link-good)

[分组示例](./ecosystem.md#ecosystem-vue-router-prefer-named-link) · [规则参考](/rules/reference/ecosystem-vue-router-prefer-named-link.md)

### `ecosystem/vue-router-prefer-named-push`

Vue Router 编程式导航优先使用具名路由对象

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

[错误示例](./ecosystem.md#ecosystem-vue-router-prefer-named-push-bad)

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

[正确示例](./ecosystem.md#ecosystem-vue-router-prefer-named-push-good)

[分组示例](./ecosystem.md#ecosystem-vue-router-prefer-named-push) · [规则参考](/rules/reference/ecosystem-vue-router-prefer-named-push.md)

### `ecosystem/vue-router-unknown-route`

名称不在完整且已安装的路由器中。

<span id="ecosystem-vue-router-unknown-route-bad"></span>

[错误示例](./cross-file.md#ecosystem-vue-router-unknown-route-bad)

<span id="ecosystem-vue-router-unknown-route-good"></span>

[正确示例](./cross-file.md#ecosystem-vue-router-unknown-route-good)

[分组示例](./cross-file.md#ecosystem-vue-router-unknown-route) · [规则参考](/rules/project/ecosystem-vue-router-unknown-route.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

[错误示例](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-bad)

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

[正确示例](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot-good)

[分组示例](./ecosystem.md#ecosystem-vue-test-utils-no-html-snapshot) · [规则参考](/rules/reference/ecosystem-vue-test-utils-no-html-snapshot.md)

### `html/cross-component-nesting`

检查导入组件组合后的实际 HTML 嵌套。

<span id="html-cross-component-nesting-bad"></span>

[错误示例](./cross-file.md#html-cross-component-nesting-bad)

<span id="html-cross-component-nesting-good"></span>

[正确示例](./cross-file.md#html-cross-component-nesting-good)

[分组示例](./cross-file.md#html-cross-component-nesting) · [规则参考](/rules/project/html-cross-component-nesting.md)

### `html/deprecated-attr`

禁止已弃用的 HTML 属性

<span id="html-deprecated-attr-bad"></span>

[错误示例](./html.md#html-deprecated-attr-bad)

<span id="html-deprecated-attr-good"></span>

[正确示例](./html.md#html-deprecated-attr-good)

[分组示例](./html.md#html-deprecated-attr) · [规则参考](/rules/reference/html-deprecated-attr.md)

### `html/deprecated-element`

禁止已弃用的 HTML 元素

<span id="html-deprecated-element-bad"></span>

[错误示例](./html.md#html-deprecated-element-bad)

<span id="html-deprecated-element-good"></span>

[正确示例](./html.md#html-deprecated-element-good)

[分组示例](./html.md#html-deprecated-element) · [规则参考](/rules/reference/html-deprecated-element.md)

### `html/id-duplication`

禁止重复的元素 ID

<span id="html-id-duplication-bad"></span>

[错误示例](./html.md#html-id-duplication-bad)

<span id="html-id-duplication-good"></span>

[正确示例](./html.md#html-id-duplication-good)

[分组示例](./html.md#html-id-duplication) · [规则参考](/rules/reference/html-id-duplication.md)

### `html/no-consecutive-br`

禁止连续的 &lt;br&gt; 元素

<span id="html-no-consecutive-br-bad"></span>

[错误示例](./html.md#html-no-consecutive-br-bad)

<span id="html-no-consecutive-br-good"></span>

[正确示例](./html.md#html-no-consecutive-br-good)

[分组示例](./html.md#html-no-consecutive-br) · [规则参考](/rules/reference/html-no-consecutive-br.md)

### `html/no-dupe-style-properties`

禁止内联 style 属性中重复的属性声明

<span id="html-no-dupe-style-properties-bad"></span>

[错误示例](./html.md#html-no-dupe-style-properties-bad)

<span id="html-no-dupe-style-properties-good"></span>

[正确示例](./html.md#html-no-dupe-style-properties-good)

[分组示例](./html.md#html-no-dupe-style-properties) · [规则参考](/rules/reference/html-no-dupe-style-properties.md)

### `html/no-duplicate-class`

禁止静态 class 属性中重复的类名

<span id="html-no-duplicate-class-bad"></span>

[错误示例](./html.md#html-no-duplicate-class-bad)

<span id="html-no-duplicate-class-good"></span>

[正确示例](./html.md#html-no-duplicate-class-good)

[分组示例](./html.md#html-no-duplicate-class) · [规则参考](/rules/reference/html-no-duplicate-class.md)

### `html/no-duplicate-dt`

禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称

<span id="html-no-duplicate-dt-bad"></span>

[错误示例](./html.md#html-no-duplicate-dt-bad)

<span id="html-no-duplicate-dt-good"></span>

[正确示例](./html.md#html-no-duplicate-dt-good)

[分组示例](./html.md#html-no-duplicate-dt) · [规则参考](/rules/reference/html-no-duplicate-dt.md)

### `html/no-empty-palpable-content`

禁止预期具有可见内容的空元素

<span id="html-no-empty-palpable-content-bad"></span>

[错误示例](./html.md#html-no-empty-palpable-content-bad)

<span id="html-no-empty-palpable-content-good"></span>

[正确示例](./html.md#html-no-empty-palpable-content-good)

[分组示例](./html.md#html-no-empty-palpable-content) · [规则参考](/rules/reference/html-no-empty-palpable-content.md)

### `html/require-datetime`

要求 &lt;time&gt; 元素具有 datetime 属性

<span id="html-require-datetime-bad"></span>

[错误示例](./html.md#html-require-datetime-bad)

<span id="html-require-datetime-good"></span>

[正确示例](./html.md#html-require-datetime-good)

[分组示例](./html.md#html-require-datetime) · [规则参考](/rules/reference/html-require-datetime.md)

### `musea/no-empty-variant`

禁止空的 &lt;variant&gt; 块

<span id="musea-no-empty-variant-bad"></span>

[错误示例](./musea-and-css.md#musea-no-empty-variant-bad)

<span id="musea-no-empty-variant-good"></span>

[正确示例](./musea-and-css.md#musea-no-empty-variant-good)

[分组示例](./musea-and-css.md#musea-no-empty-variant) · [规则参考](/rules/reference/musea-no-empty-variant.md)

### `musea/prefer-design-tokens`

优先使用设计令牌 CSS 变量，而不是硬编码的原始值

<span id="musea-prefer-design-tokens-bad"></span>

[错误示例](./musea-and-css.md#musea-prefer-design-tokens-bad)

<span id="musea-prefer-design-tokens-good"></span>

[正确示例](./musea-and-css.md#musea-prefer-design-tokens-good)

[分组示例](./musea-and-css.md#musea-prefer-design-tokens) · [规则参考](/rules/reference/musea-prefer-design-tokens.md)

### `musea/require-component`

要求 &lt;art&gt; 块具有 component 属性

<span id="musea-require-component-bad"></span>

[错误示例](./musea-and-css.md#musea-require-component-bad)

<span id="musea-require-component-good"></span>

[正确示例](./musea-and-css.md#musea-require-component-good)

[分组示例](./musea-and-css.md#musea-require-component) · [规则参考](/rules/reference/musea-require-component.md)

### `musea/require-title`

要求 &lt;art&gt; 块具有 title 属性

<span id="musea-require-title-bad"></span>

[错误示例](./musea-and-css.md#musea-require-title-bad)

<span id="musea-require-title-good"></span>

[正确示例](./musea-and-css.md#musea-require-title-good)

[分组示例](./musea-and-css.md#musea-require-title) · [规则参考](/rules/reference/musea-require-title.md)

### `musea/unique-variant-names`

要求 variant 名称唯一

<span id="musea-unique-variant-names-bad"></span>

[错误示例](./musea-and-css.md#musea-unique-variant-names-bad)

<span id="musea-unique-variant-names-good"></span>

[正确示例](./musea-and-css.md#musea-unique-variant-names-good)

[分组示例](./musea-and-css.md#musea-unique-variant-names) · [规则参考](/rules/reference/musea-unique-variant-names.md)

### `musea/valid-variant`

要求 &lt;variant&gt; 块具有 name 属性

<span id="musea-valid-variant-bad"></span>

[错误示例](./musea-and-css.md#musea-valid-variant-bad)

<span id="musea-valid-variant-good"></span>

[正确示例](./musea-and-css.md#musea-valid-variant-good)

[分组示例](./musea-and-css.md#musea-valid-variant) · [规则参考](/rules/reference/musea-valid-variant.md)

### `nuxt/no-nuxt-config-test-key`

禁止在 Nuxt 配置中设置 `test` 键

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

[错误示例](./ecosystem.md#nuxt-no-nuxt-config-test-key-bad)

<span id="nuxt-no-nuxt-config-test-key-good"></span>

[正确示例](./ecosystem.md#nuxt-no-nuxt-config-test-key-good)

[分组示例](./ecosystem.md#nuxt-no-nuxt-config-test-key) · [规则参考](/rules/reference/nuxt-no-nuxt-config-test-key.md)

### `nuxt/no-page-meta-runtime-values`

禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

[错误示例](./ecosystem.md#nuxt-no-page-meta-runtime-values-bad)

<span id="nuxt-no-page-meta-runtime-values-good"></span>

[正确示例](./ecosystem.md#nuxt-no-page-meta-runtime-values-good)

[分组示例](./ecosystem.md#nuxt-no-page-meta-runtime-values) · [规则参考](/rules/reference/nuxt-no-page-meta-runtime-values.md)

### `nuxt/nuxt-config-keys-order`

优先使用推荐的 Nuxt 配置属性顺序

<span id="nuxt-nuxt-config-keys-order-bad"></span>

[错误示例](./ecosystem.md#nuxt-nuxt-config-keys-order-bad)

<span id="nuxt-nuxt-config-keys-order-good"></span>

[正确示例](./ecosystem.md#nuxt-nuxt-config-keys-order-good)

[分组示例](./ecosystem.md#nuxt-nuxt-config-keys-order) · [规则参考](/rules/reference/nuxt-nuxt-config-keys-order.md)

### `nuxt/prefer-import-meta`

优先使用 `import.meta.*`，而不是 `process.*`

<span id="nuxt-prefer-import-meta-bad"></span>

[错误示例](./ecosystem.md#nuxt-prefer-import-meta-bad)

<span id="nuxt-prefer-import-meta-good"></span>

[正确示例](./ecosystem.md#nuxt-prefer-import-meta-good)

[分组示例](./ecosystem.md#nuxt-prefer-import-meta) · [规则参考](/rules/reference/nuxt-prefer-import-meta.md)

### `petite-vue/no-unsupported-directive`

禁止 petite-vue 不支持的指令

<span id="petite-vue-no-unsupported-directive-bad"></span>

[错误示例](./petite-vue.md#petite-vue-no-unsupported-directive-bad)

<span id="petite-vue-no-unsupported-directive-good"></span>

[正确示例](./petite-vue.md#petite-vue-no-unsupported-directive-good)

[分组示例](./petite-vue.md#petite-vue-no-unsupported-directive) · [规则参考](/rules/reference/petite-vue-no-unsupported-directive.md)

### `petite-vue/valid-v-effect`

要求 v-effect 具有非空表达式

<span id="petite-vue-valid-v-effect-bad"></span>

[错误示例](./petite-vue.md#petite-vue-valid-v-effect-bad)

<span id="petite-vue-valid-v-effect-good"></span>

[正确示例](./petite-vue.md#petite-vue-valid-v-effect-good)

[分组示例](./petite-vue.md#petite-vue-valid-v-effect) · [规则参考](/rules/reference/petite-vue-valid-v-effect.md)

### `petite-vue/valid-v-scope`

要求 v-scope 绑定对象字面量

<span id="petite-vue-valid-v-scope-bad"></span>

[错误示例](./petite-vue.md#petite-vue-valid-v-scope-bad)

<span id="petite-vue-valid-v-scope-good"></span>

[正确示例](./petite-vue.md#petite-vue-valid-v-scope-good)

[分组示例](./petite-vue.md#petite-vue-valid-v-scope) · [规则参考](/rules/reference/petite-vue-valid-v-scope.md)

### `script/component-options-name-casing`

要求组件 `name` 选项使用 PascalCase

<span id="script-component-options-name-casing-bad"></span>

[错误示例](./type-and-script.md#script-component-options-name-casing-bad)

<span id="script-component-options-name-casing-good"></span>

[正确示例](./type-and-script.md#script-component-options-name-casing-good)

[分组示例](./type-and-script.md#script-component-options-name-casing) · [规则参考](/rules/reference/script-component-options-name-casing.md)

### `script/custom-event-name-casing`

要求发出的自定义事件名称使用 camelCase

<span id="script-custom-event-name-casing-bad"></span>

[错误示例](./type-and-script.md#script-custom-event-name-casing-bad)

<span id="script-custom-event-name-casing-good"></span>

[正确示例](./type-and-script.md#script-custom-event-name-casing-good)

[分组示例](./type-and-script.md#script-custom-event-name-casing) · [规则参考](/rules/reference/script-custom-event-name-casing.md)

### `script/define-emits-declaration`

要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式

<span id="script-define-emits-declaration-bad"></span>

[错误示例](./type-and-script.md#script-define-emits-declaration-bad)

<span id="script-define-emits-declaration-good"></span>

[正确示例](./type-and-script.md#script-define-emits-declaration-good)

[分组示例](./type-and-script.md#script-define-emits-declaration) · [规则参考](/rules/reference/script-define-emits-declaration.md)

### `script/define-macros-order`

要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序

<span id="script-define-macros-order-bad"></span>

[错误示例](./type-and-script.md#script-define-macros-order-bad)

<span id="script-define-macros-order-good"></span>

[正确示例](./type-and-script.md#script-define-macros-order-good)

[分组示例](./type-and-script.md#script-define-macros-order) · [规则参考](/rules/reference/script-define-macros-order.md)

### `script/define-props-declaration`

要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式

<span id="script-define-props-declaration-bad"></span>

[错误示例](./type-and-script.md#script-define-props-declaration-bad)

<span id="script-define-props-declaration-good"></span>

[正确示例](./type-and-script.md#script-define-props-declaration-good)

[分组示例](./type-and-script.md#script-define-props-declaration) · [规则参考](/rules/reference/script-define-props-declaration.md)

### `script/define-props-destructuring`

要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致

<span id="script-define-props-destructuring-bad"></span>

[错误示例](./type-and-script.md#script-define-props-destructuring-bad)

<span id="script-define-props-destructuring-good"></span>

[正确示例](./type-and-script.md#script-define-props-destructuring-good)

[分组示例](./type-and-script.md#script-define-props-destructuring) · [规则参考](/rules/reference/script-define-props-destructuring.md)

### `script/no-arrow-functions-in-watch`

禁止将箭头函数用作 Options API watch 处理器

<span id="script-no-arrow-functions-in-watch-bad"></span>

[错误示例](./type-and-script.md#script-no-arrow-functions-in-watch-bad)

<span id="script-no-arrow-functions-in-watch-good"></span>

[正确示例](./type-and-script.md#script-no-arrow-functions-in-watch-good)

[分组示例](./type-and-script.md#script-no-arrow-functions-in-watch) · [规则参考](/rules/reference/script-no-arrow-functions-in-watch.md)

### `script/no-async-in-computed`

禁止计算属性中的异步函数

<span id="script-no-async-in-computed-bad"></span>

[错误示例](./type-and-script.md#script-no-async-in-computed-bad)

<span id="script-no-async-in-computed-good"></span>

[正确示例](./type-and-script.md#script-no-async-in-computed-good)

[分组示例](./type-and-script.md#script-no-async-in-computed) · [规则参考](/rules/reference/script-no-async-in-computed.md)

### `script/no-boolean-default`

禁止为 Boolean prop 设置默认值

<span id="script-no-boolean-default-bad"></span>

[错误示例](./type-and-script.md#script-no-boolean-default-bad)

<span id="script-no-boolean-default-good"></span>

[正确示例](./type-and-script.md#script-no-boolean-default-good)

[分组示例](./type-and-script.md#script-no-boolean-default) · [规则参考](/rules/reference/script-no-boolean-default.md)

### `script/no-deep-destructure-in-props`

禁止在 defineProps 中进行深层嵌套解构

<span id="script-no-deep-destructure-in-props-bad"></span>

[错误示例](./type-and-script.md#script-no-deep-destructure-in-props-bad)

<span id="script-no-deep-destructure-in-props-good"></span>

[正确示例](./type-and-script.md#script-no-deep-destructure-in-props-good)

[分组示例](./type-and-script.md#script-no-deep-destructure-in-props) · [规则参考](/rules/reference/script-no-deep-destructure-in-props.md)

### `script/no-deprecated-data-object-declaration`

禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）

<span id="script-no-deprecated-data-object-declaration-bad"></span>

[错误示例](./type-and-script.md#script-no-deprecated-data-object-declaration-bad)

<span id="script-no-deprecated-data-object-declaration-good"></span>

[正确示例](./type-and-script.md#script-no-deprecated-data-object-declaration-good)

[分组示例](./type-and-script.md#script-no-deprecated-data-object-declaration) · [规则参考](/rules/reference/script-no-deprecated-data-object-declaration.md)

### `script/no-deprecated-destroyed-lifecycle`

禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

[错误示例](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-bad)

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

[正确示例](./type-and-script.md#script-no-deprecated-destroyed-lifecycle-good)

[分组示例](./type-and-script.md#script-no-deprecated-destroyed-lifecycle) · [规则参考](/rules/reference/script-no-deprecated-destroyed-lifecycle.md)

### `script/no-deprecated-dollar-listeners-api`

禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

[错误示例](./type-and-script.md#script-no-deprecated-dollar-listeners-api-bad)

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

[正确示例](./type-and-script.md#script-no-deprecated-dollar-listeners-api-good)

[分组示例](./type-and-script.md#script-no-deprecated-dollar-listeners-api) · [规则参考](/rules/reference/script-no-deprecated-dollar-listeners-api.md)

### `script/no-deprecated-dollar-scopedslots-api`

禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

[错误示例](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-bad)

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

[正确示例](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api-good)

[分组示例](./type-and-script.md#script-no-deprecated-dollar-scopedslots-api) · [规则参考](/rules/reference/script-no-deprecated-dollar-scopedslots-api.md)

### `script/no-deprecated-events-api`

禁止已移除的 Vue 2 事件 API（$on / $off / $once）

<span id="script-no-deprecated-events-api-bad"></span>

[错误示例](./type-and-script.md#script-no-deprecated-events-api-bad)

<span id="script-no-deprecated-events-api-good"></span>

[正确示例](./type-and-script.md#script-no-deprecated-events-api-good)

[分组示例](./type-and-script.md#script-no-deprecated-events-api) · [规则参考](/rules/reference/script-no-deprecated-events-api.md)

### `script/no-deprecated-props-default-this`

禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）

<span id="script-no-deprecated-props-default-this-bad"></span>

[错误示例](./type-and-script.md#script-no-deprecated-props-default-this-bad)

<span id="script-no-deprecated-props-default-this-good"></span>

[正确示例](./type-and-script.md#script-no-deprecated-props-default-this-good)

[分组示例](./type-and-script.md#script-no-deprecated-props-default-this) · [规则参考](/rules/reference/script-no-deprecated-props-default-this.md)

### `script/no-dupe-keys`

禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键

<span id="script-no-dupe-keys-bad"></span>

[错误示例](./type-and-script.md#script-no-dupe-keys-bad)

<span id="script-no-dupe-keys-good"></span>

[正确示例](./type-and-script.md#script-no-dupe-keys-good)

[分组示例](./type-and-script.md#script-no-dupe-keys) · [规则参考](/rules/reference/script-no-dupe-keys.md)

### `script/no-duplicate-attr-inheritance`

标记重复应用透传属性的组件

<span id="script-no-duplicate-attr-inheritance-bad"></span>

[错误示例](./type-and-script.md#script-no-duplicate-attr-inheritance-bad)

<span id="script-no-duplicate-attr-inheritance-good"></span>

[正确示例](./type-and-script.md#script-no-duplicate-attr-inheritance-good)

[分组示例](./type-and-script.md#script-no-duplicate-attr-inheritance) · [规则参考](/rules/reference/script-no-duplicate-attr-inheritance.md)

### `script/no-export-in-script-setup`

禁止 &lt;script setup&gt; 内的 export 语句

<span id="script-no-export-in-script-setup-bad"></span>

[错误示例](./type-and-script.md#script-no-export-in-script-setup-bad)

<span id="script-no-export-in-script-setup-good"></span>

[正确示例](./type-and-script.md#script-no-export-in-script-setup-good)

[分组示例](./type-and-script.md#script-no-export-in-script-setup) · [规则参考](/rules/reference/script-no-export-in-script-setup.md)

### `script/no-expose-after-await`

禁止在 await 之后调用 expose() 和 defineExpose()

<span id="script-no-expose-after-await-bad"></span>

[错误示例](./type-and-script.md#script-no-expose-after-await-bad)

<span id="script-no-expose-after-await-good"></span>

[正确示例](./type-and-script.md#script-no-expose-after-await-good)

[分组示例](./type-and-script.md#script-no-expose-after-await) · [规则参考](/rules/reference/script-no-expose-after-await.md)

### `script/no-get-current-instance`

禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）

<span id="script-no-get-current-instance-bad"></span>

[错误示例](./type-and-script.md#script-no-get-current-instance-bad)

<span id="script-no-get-current-instance-good"></span>

[正确示例](./type-and-script.md#script-no-get-current-instance-good)

[分组示例](./type-and-script.md#script-no-get-current-instance) · [规则参考](/rules/reference/script-no-get-current-instance.md)

### `script/no-import-compiler-macros`

禁止导入自动可用的 Vue 编译器宏

<span id="script-no-import-compiler-macros-bad"></span>

[错误示例](./type-and-script.md#script-no-import-compiler-macros-bad)

<span id="script-no-import-compiler-macros-good"></span>

[正确示例](./type-and-script.md#script-no-import-compiler-macros-good)

[分组示例](./type-and-script.md#script-no-import-compiler-macros) · [规则参考](/rules/reference/script-no-import-compiler-macros.md)

### `script/no-internal-imports`

禁止从 Vue 内部模块导入

<span id="script-no-internal-imports-bad"></span>

[错误示例](./type-and-script.md#script-no-internal-imports-bad)

<span id="script-no-internal-imports-good"></span>

[正确示例](./type-and-script.md#script-no-internal-imports-good)

[分组示例](./type-and-script.md#script-no-internal-imports) · [规则参考](/rules/reference/script-no-internal-imports.md)

### `script/no-lifecycle-after-await`

禁止在 setup() 中 await 之后注册生命周期钩子

<span id="script-no-lifecycle-after-await-bad"></span>

[错误示例](./type-and-script.md#script-no-lifecycle-after-await-bad)

<span id="script-no-lifecycle-after-await-good"></span>

[正确示例](./type-and-script.md#script-no-lifecycle-after-await-good)

[分组示例](./type-and-script.md#script-no-lifecycle-after-await) · [规则参考](/rules/reference/script-no-lifecycle-after-await.md)

### `script/no-multiple-slot-args`

禁止向作用域插槽函数调用传入多个参数

<span id="script-no-multiple-slot-args-bad"></span>

[错误示例](./type-and-script.md#script-no-multiple-slot-args-bad)

<span id="script-no-multiple-slot-args-good"></span>

[正确示例](./type-and-script.md#script-no-multiple-slot-args-good)

[分组示例](./type-and-script.md#script-no-multiple-slot-args) · [规则参考](/rules/reference/script-no-multiple-slot-args.md)

### `script/no-next-tick`

禁止面向 Vapor 的组件使用 nextTick()

<span id="script-no-next-tick-bad"></span>

[错误示例](./type-and-script.md#script-no-next-tick-bad)

<span id="script-no-next-tick-good"></span>

[正确示例](./type-and-script.md#script-no-next-tick-good)

[分组示例](./type-and-script.md#script-no-next-tick) · [规则参考](/rules/reference/script-no-next-tick.md)

### `script/no-options-api`

禁止 Vapor 模式中的 Options API 模式

<span id="script-no-options-api-bad"></span>

[错误示例](./type-and-script.md#script-no-options-api-bad)

<span id="script-no-options-api-good"></span>

[正确示例](./type-and-script.md#script-no-options-api-good)

[分组示例](./type-and-script.md#script-no-options-api) · [规则参考](/rules/reference/script-no-options-api.md)

### `script/no-potential-component-option-typo`

标记 Options API 组件选项名称中的疑似拼写错误

<span id="script-no-potential-component-option-typo-bad"></span>

[错误示例](./type-and-script.md#script-no-potential-component-option-typo-bad)

<span id="script-no-potential-component-option-typo-good"></span>

[正确示例](./type-and-script.md#script-no-potential-component-option-typo-good)

[分组示例](./type-and-script.md#script-no-potential-component-option-typo) · [规则参考](/rules/reference/script-no-potential-component-option-typo.md)

### `script/no-reactive-destructure`

禁止导致响应性丢失的响应式对象解构

<span id="script-no-reactive-destructure-bad"></span>

[错误示例](./type-and-script.md#script-no-reactive-destructure-bad)

<span id="script-no-reactive-destructure-good"></span>

[正确示例](./type-and-script.md#script-no-reactive-destructure-good)

[分组示例](./type-and-script.md#script-no-reactive-destructure) · [规则参考](/rules/reference/script-no-reactive-destructure.md)

### `script/no-ref-as-operand`

要求将 ref 绑定变量用作操作数时通过 `.value` 访问

<span id="script-no-ref-as-operand-bad"></span>

[错误示例](./type-and-script.md#script-no-ref-as-operand-bad)

<span id="script-no-ref-as-operand-good"></span>

[正确示例](./type-and-script.md#script-no-ref-as-operand-good)

[分组示例](./type-and-script.md#script-no-ref-as-operand) · [规则参考](/rules/reference/script-no-ref-as-operand.md)

### `script/no-required-prop-with-default`

禁止 prop 同时具有 required: true 和默认值

<span id="script-no-required-prop-with-default-bad"></span>

[错误示例](./type-and-script.md#script-no-required-prop-with-default-bad)

<span id="script-no-required-prop-with-default-good"></span>

[正确示例](./type-and-script.md#script-no-required-prop-with-default-good)

[分组示例](./type-and-script.md#script-no-required-prop-with-default) · [规则参考](/rules/reference/script-no-required-prop-with-default.md)

### `script/no-reserved-identifiers`

禁止使用 Vue 编译器保留标识符

<span id="script-no-reserved-identifiers-bad"></span>

[错误示例](./type-and-script.md#script-no-reserved-identifiers-bad)

<span id="script-no-reserved-identifiers-good"></span>

[正确示例](./type-and-script.md#script-no-reserved-identifiers-good)

[分组示例](./type-and-script.md#script-no-reserved-identifiers) · [规则参考](/rules/reference/script-no-reserved-identifiers.md)

### `script/no-reserved-keys`

禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键

<span id="script-no-reserved-keys-bad"></span>

[错误示例](./type-and-script.md#script-no-reserved-keys-bad)

<span id="script-no-reserved-keys-good"></span>

[正确示例](./type-and-script.md#script-no-reserved-keys-good)

[分组示例](./type-and-script.md#script-no-reserved-keys) · [规则参考](/rules/reference/script-no-reserved-keys.md)

### `script/no-reserved-props`

禁止在组件 props 声明中使用保留名称

<span id="script-no-reserved-props-bad"></span>

[错误示例](./type-and-script.md#script-no-reserved-props-bad)

<span id="script-no-reserved-props-good"></span>

[正确示例](./type-and-script.md#script-no-reserved-props-good)

[分组示例](./type-and-script.md#script-no-reserved-props) · [规则参考](/rules/reference/script-no-reserved-props.md)

### `script/no-restricted-globals`

禁止直接引用必须通过类型化包装器访问的运行时环境全局变量

<span id="script-no-restricted-globals-bad"></span>

[错误示例](./type-and-script.md#script-no-restricted-globals-bad)

<span id="script-no-restricted-globals-good"></span>

[正确示例](./type-and-script.md#script-no-restricted-globals-good)

[分组示例](./type-and-script.md#script-no-restricted-globals) · [规则参考](/rules/reference/script-no-restricted-globals.md)

### `script/no-restricted-members`

禁止项目配置的 object.property 成员访问

<span id="script-no-restricted-members-bad"></span>

[错误示例](./type-and-script.md#script-no-restricted-members-bad)

<span id="script-no-restricted-members-good"></span>

[正确示例](./type-and-script.md#script-no-restricted-members-good)

[分组示例](./type-and-script.md#script-no-restricted-members) · [规则参考](/rules/reference/script-no-restricted-members.md)

### `script/no-side-effects-in-computed-properties`

禁止 Options API 计算 getter 中的副作用

<span id="script-no-side-effects-in-computed-properties-bad"></span>

[错误示例](./type-and-script.md#script-no-side-effects-in-computed-properties-bad)

<span id="script-no-side-effects-in-computed-properties-good"></span>

[正确示例](./type-and-script.md#script-no-side-effects-in-computed-properties-good)

[分组示例](./type-and-script.md#script-no-side-effects-in-computed-properties) · [规则参考](/rules/reference/script-no-side-effects-in-computed-properties.md)

### `script/no-top-level-ref-in-script`

禁止顶层 ref/reactive，以防止跨请求状态污染

<span id="script-no-top-level-ref-in-script-bad"></span>

[错误示例](./type-and-script.md#script-no-top-level-ref-in-script-bad)

<span id="script-no-top-level-ref-in-script-good"></span>

[正确示例](./type-and-script.md#script-no-top-level-ref-in-script-good)

[分组示例](./type-and-script.md#script-no-top-level-ref-in-script) · [规则参考](/rules/reference/script-no-top-level-ref-in-script.md)

### `script/no-unstable-nested-components`

禁止在 setup 或 render 函数内定义组件

<span id="script-no-unstable-nested-components-bad"></span>

[错误示例](./type-and-script.md#script-no-unstable-nested-components-bad)

<span id="script-no-unstable-nested-components-good"></span>

[正确示例](./type-and-script.md#script-no-unstable-nested-components-good)

[分组示例](./type-and-script.md#script-no-unstable-nested-components) · [规则参考](/rules/reference/script-no-unstable-nested-components.md)

### `script/no-unused-emit-declarations`

标记已声明却从未发出的事件

<span id="script-no-unused-emit-declarations-bad"></span>

[错误示例](./type-and-script.md#script-no-unused-emit-declarations-bad)

<span id="script-no-unused-emit-declarations-good"></span>

[正确示例](./type-and-script.md#script-no-unused-emit-declarations-good)

[分组示例](./type-and-script.md#script-no-unused-emit-declarations) · [规则参考](/rules/reference/script-no-unused-emit-declarations.md)

### `script/no-use-computed-property-like-method`

禁止像方法一样调用 Options API 计算属性

<span id="script-no-use-computed-property-like-method-bad"></span>

[错误示例](./type-and-script.md#script-no-use-computed-property-like-method-bad)

<span id="script-no-use-computed-property-like-method-good"></span>

[正确示例](./type-and-script.md#script-no-use-computed-property-like-method-good)

[分组示例](./type-and-script.md#script-no-use-computed-property-like-method) · [规则参考](/rules/reference/script-no-use-computed-property-like-method.md)

### `script/no-watch-after-await`

禁止在 setup() 中 await 之后注册未保留停止句柄的侦听器

<span id="script-no-watch-after-await-bad"></span>

[错误示例](./type-and-script.md#script-no-watch-after-await-bad)

<span id="script-no-watch-after-await-good"></span>

[正确示例](./type-and-script.md#script-no-watch-after-await-good)

[分组示例](./type-and-script.md#script-no-watch-after-await) · [规则参考](/rules/reference/script-no-watch-after-await.md)

### `script/no-with-defaults`

不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）

<span id="script-no-with-defaults-bad"></span>

[错误示例](./type-and-script.md#script-no-with-defaults-bad)

<span id="script-no-with-defaults-good"></span>

[正确示例](./type-and-script.md#script-no-with-defaults-good)

[分组示例](./type-and-script.md#script-no-with-defaults) · [规则参考](/rules/reference/script-no-with-defaults.md)

### `script/prefer-computed`

响应式派生状态优先使用 computed()

<span id="script-prefer-computed-bad"></span>

[错误示例](./type-and-script.md#script-prefer-computed-bad)

<span id="script-prefer-computed-good"></span>

[正确示例](./type-and-script.md#script-prefer-computed-good)

[分组示例](./type-and-script.md#script-prefer-computed) · [规则参考](/rules/reference/script-prefer-computed.md)

### `script/prefer-define-options`

优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;

<span id="script-prefer-define-options-bad"></span>

[错误示例](./type-and-script.md#script-prefer-define-options-bad)

<span id="script-prefer-define-options-good"></span>

[正确示例](./type-and-script.md#script-prefer-define-options-good)

[分组示例](./type-and-script.md#script-prefer-define-options) · [规则参考](/rules/reference/script-prefer-define-options.md)

### `script/prefer-import-from-vue`

优先从 'vue' 导入，而不是内部包

<span id="script-prefer-import-from-vue-bad"></span>

[错误示例](./type-and-script.md#script-prefer-import-from-vue-bad)

<span id="script-prefer-import-from-vue-good"></span>

[正确示例](./type-and-script.md#script-prefer-import-from-vue-good)

[分组示例](./type-and-script.md#script-prefer-import-from-vue) · [规则参考](/rules/reference/script-prefer-import-from-vue.md)

### `script/prefer-ref-over-reactive`

建议使用 ref() 而不是 reactive() 管理状态

<span id="script-prefer-ref-over-reactive-bad"></span>

[错误示例](./type-and-script.md#script-prefer-ref-over-reactive-bad)

<span id="script-prefer-ref-over-reactive-good"></span>

[正确示例](./type-and-script.md#script-prefer-ref-over-reactive-good)

[分组示例](./type-and-script.md#script-prefer-ref-over-reactive) · [规则参考](/rules/reference/script-prefer-ref-over-reactive.md)

### `script/prefer-use-attrs`

建议使用 useAttrs() 而不是 context.attrs

<span id="script-prefer-use-attrs-bad"></span>

[错误示例](./type-and-script.md#script-prefer-use-attrs-bad)

<span id="script-prefer-use-attrs-good"></span>

[正确示例](./type-and-script.md#script-prefer-use-attrs-good)

[分组示例](./type-and-script.md#script-prefer-use-attrs) · [规则参考](/rules/reference/script-prefer-use-attrs.md)

### `script/prefer-use-id`

建议使用 useId() 生成唯一 ID（Vue 3.5+）

<span id="script-prefer-use-id-bad"></span>

[错误示例](./type-and-script.md#script-prefer-use-id-bad)

<span id="script-prefer-use-id-good"></span>

[正确示例](./type-and-script.md#script-prefer-use-id-good)

[分组示例](./type-and-script.md#script-prefer-use-id) · [规则参考](/rules/reference/script-prefer-use-id.md)

### `script/prefer-use-slots`

建议使用 useSlots() 而不是 context.slots

<span id="script-prefer-use-slots-bad"></span>

[错误示例](./type-and-script.md#script-prefer-use-slots-bad)

<span id="script-prefer-use-slots-good"></span>

[正确示例](./type-and-script.md#script-prefer-use-slots-good)

[分组示例](./type-and-script.md#script-prefer-use-slots) · [规则参考](/rules/reference/script-prefer-use-slots.md)

### `script/prefer-use-template-ref`

模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+）

<span id="script-prefer-use-template-ref-bad"></span>

[错误示例](./type-and-script.md#script-prefer-use-template-ref-bad)

<span id="script-prefer-use-template-ref-good"></span>

[正确示例](./type-and-script.md#script-prefer-use-template-ref-good)

[分组示例](./type-and-script.md#script-prefer-use-template-ref) · [规则参考](/rules/reference/script-prefer-use-template-ref.md)

### `script/require-default-prop`

要求每个可选的非 Boolean prop 具有默认值

<span id="script-require-default-prop-bad"></span>

[错误示例](./type-and-script.md#script-require-default-prop-bad)

<span id="script-require-default-prop-good"></span>

[正确示例](./type-and-script.md#script-require-default-prop-good)

[分组示例](./type-and-script.md#script-require-default-prop) · [规则参考](/rules/reference/script-require-default-prop.md)

### `script/require-explicit-emits`

要求发出的事件在 defineEmits 或 emits 选项中声明

<span id="script-require-explicit-emits-bad"></span>

[错误示例](./type-and-script.md#script-require-explicit-emits-bad)

<span id="script-require-explicit-emits-good"></span>

[正确示例](./type-and-script.md#script-require-explicit-emits-good)

[分组示例](./type-and-script.md#script-require-explicit-emits) · [规则参考](/rules/reference/script-require-explicit-emits.md)

### `script/require-explicit-slots`

要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型

<span id="script-require-explicit-slots-bad"></span>

[错误示例](./type-and-script.md#script-require-explicit-slots-bad)

<span id="script-require-explicit-slots-good"></span>

[正确示例](./type-and-script.md#script-require-explicit-slots-good)

[分组示例](./type-and-script.md#script-require-explicit-slots) · [规则参考](/rules/reference/script-require-explicit-slots.md)

### `script/require-function-return-type`

要求函数具有返回类型注解

<span id="script-require-function-return-type-bad"></span>

[错误示例](./type-and-script.md#script-require-function-return-type-bad)

<span id="script-require-function-return-type-good"></span>

[正确示例](./type-and-script.md#script-require-function-return-type-good)

[分组示例](./type-and-script.md#script-require-function-return-type) · [规则参考](/rules/reference/script-require-function-return-type.md)

### `script/require-prop-type-constructor`

要求 prop 的 `type` 值是构造器，而不是字符串字面量

<span id="script-require-prop-type-constructor-bad"></span>

[错误示例](./type-and-script.md#script-require-prop-type-constructor-bad)

<span id="script-require-prop-type-constructor-good"></span>

[正确示例](./type-and-script.md#script-require-prop-type-constructor-good)

[分组示例](./type-and-script.md#script-require-prop-type-constructor) · [规则参考](/rules/reference/script-require-prop-type-constructor.md)

### `script/require-prop-types`

要求每个 prop 声明类型

<span id="script-require-prop-types-bad"></span>

[错误示例](./type-and-script.md#script-require-prop-types-bad)

<span id="script-require-prop-types-good"></span>

[正确示例](./type-and-script.md#script-require-prop-types-good)

[分组示例](./type-and-script.md#script-require-prop-types) · [规则参考](/rules/reference/script-require-prop-types.md)

### `script/require-symbol-provide`

建议使用 Symbol 作为 provide/inject 的注入键

<span id="script-require-symbol-provide-bad"></span>

[错误示例](./type-and-script.md#script-require-symbol-provide-bad)

<span id="script-require-symbol-provide-good"></span>

[正确示例](./type-and-script.md#script-require-symbol-provide-good)

[分组示例](./type-and-script.md#script-require-symbol-provide) · [规则参考](/rules/reference/script-require-symbol-provide.md)

### `script/require-typed-object-prop`

要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型

<span id="script-require-typed-object-prop-bad"></span>

[错误示例](./type-and-script.md#script-require-typed-object-prop-bad)

<span id="script-require-typed-object-prop-good"></span>

[正确示例](./type-and-script.md#script-require-typed-object-prop-good)

[分组示例](./type-and-script.md#script-require-typed-object-prop) · [规则参考](/rules/reference/script-require-typed-object-prop.md)

### `script/require-typed-ref`

要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数

<span id="script-require-typed-ref-bad"></span>

[错误示例](./type-and-script.md#script-require-typed-ref-bad)

<span id="script-require-typed-ref-good"></span>

[正确示例](./type-and-script.md#script-require-typed-ref-good)

[分组示例](./type-and-script.md#script-require-typed-ref) · [规则参考](/rules/reference/script-require-typed-ref.md)

### `script/require-valid-default-prop`

要求 prop 默认值符合声明类型

<span id="script-require-valid-default-prop-bad"></span>

[错误示例](./type-and-script.md#script-require-valid-default-prop-bad)

<span id="script-require-valid-default-prop-good"></span>

[正确示例](./type-and-script.md#script-require-valid-default-prop-good)

[分组示例](./type-and-script.md#script-require-valid-default-prop) · [规则参考](/rules/reference/script-require-valid-default-prop.md)

### `script/return-in-computed-property`

要求每个计算 getter 返回值

<span id="script-return-in-computed-property-bad"></span>

[错误示例](./type-and-script.md#script-return-in-computed-property-bad)

<span id="script-return-in-computed-property-good"></span>

[正确示例](./type-and-script.md#script-return-in-computed-property-good)

[分组示例](./type-and-script.md#script-return-in-computed-property) · [规则参考](/rules/reference/script-return-in-computed-property.md)

### `script/return-in-emits-validator`

要求每个 Options API emits 校验函数返回值

<span id="script-return-in-emits-validator-bad"></span>

[错误示例](./type-and-script.md#script-return-in-emits-validator-bad)

<span id="script-return-in-emits-validator-good"></span>

[正确示例](./type-and-script.md#script-return-in-emits-validator-good)

[分组示例](./type-and-script.md#script-return-in-emits-validator) · [规则参考](/rules/reference/script-return-in-emits-validator.md)

### `script/valid-define-emits`

要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）

<span id="script-valid-define-emits-bad"></span>

[错误示例](./type-and-script.md#script-valid-define-emits-bad)

<span id="script-valid-define-emits-good"></span>

[正确示例](./type-and-script.md#script-valid-define-emits-good)

[分组示例](./type-and-script.md#script-valid-define-emits) · [规则参考](/rules/reference/script-valid-define-emits.md)

### `script/valid-define-options`

要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots）

<span id="script-valid-define-options-bad"></span>

[错误示例](./type-and-script.md#script-valid-define-options-bad)

<span id="script-valid-define-options-good"></span>

[正确示例](./type-and-script.md#script-valid-define-options-good)

[分组示例](./type-and-script.md#script-valid-define-options) · [规则参考](/rules/reference/script-valid-define-options.md)

### `script/valid-define-props`

要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）

<span id="script-valid-define-props-bad"></span>

[错误示例](./type-and-script.md#script-valid-define-props-bad)

<span id="script-valid-define-props-good"></span>

[正确示例](./type-and-script.md#script-valid-define-props-good)

[分组示例](./type-and-script.md#script-valid-define-props) · [规则参考](/rules/reference/script-valid-define-props.md)

### `script/valid-next-tick`

要求 nextTick() 调用结果被等待、链式处理，或提供回调

<span id="script-valid-next-tick-bad"></span>

[错误示例](./type-and-script.md#script-valid-next-tick-bad)

<span id="script-valid-next-tick-good"></span>

[正确示例](./type-and-script.md#script-valid-next-tick-good)

[分组示例](./type-and-script.md#script-valid-next-tick) · [规则参考](/rules/reference/script-valid-next-tick.md)

### `ssr/no-browser-globals-in-ssr`

禁止 SSR 上下文中的浏览器专用全局变量

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

[错误示例](./ssr.md#ssr-no-browser-globals-in-ssr-bad)

<span id="ssr-no-browser-globals-in-ssr-good"></span>

[正确示例](./ssr.md#ssr-no-browser-globals-in-ssr-good)

[分组示例](./ssr.md#ssr-no-browser-globals-in-ssr) · [规则参考](/rules/reference/ssr-no-browser-globals-in-ssr.md)

### `ssr/no-hydration-mismatch`

禁止导致水合不匹配的不确定值

<span id="ssr-no-hydration-mismatch-bad"></span>

[错误示例](./ssr.md#ssr-no-hydration-mismatch-bad)

<span id="ssr-no-hydration-mismatch-good"></span>

[正确示例](./ssr.md#ssr-no-hydration-mismatch-good)

[分组示例](./ssr.md#ssr-no-hydration-mismatch) · [规则参考](/rules/reference/ssr-no-hydration-mismatch.md)

### `type/no-floating-promises`

禁止悬空（未处理）的 Promise

<span id="type-no-floating-promises-bad"></span>

[错误示例](./type-and-script.md#type-no-floating-promises-bad)

<span id="type-no-floating-promises-good"></span>

[正确示例](./type-and-script.md#type-no-floating-promises-good)

[分组示例](./type-and-script.md#type-no-floating-promises) · [规则参考](/rules/reference/type-no-floating-promises.md)

### `type/no-reactivity-loss`

禁止赋值和调用中对响应式值取普通快照

<span id="type-no-reactivity-loss-bad"></span>

[错误示例](./type-and-script.md#type-no-reactivity-loss-bad)

<span id="type-no-reactivity-loss-good"></span>

[正确示例](./type-and-script.md#type-no-reactivity-loss-good)

[分组示例](./type-and-script.md#type-no-reactivity-loss) · [规则参考](/rules/reference/type-no-reactivity-loss.md)

### `type/no-unsafe-template-binding`

禁止解析为不安全类型的模板绑定

<span id="type-no-unsafe-template-binding-bad"></span>

[错误示例](./type-and-script.md#type-no-unsafe-template-binding-bad)

<span id="type-no-unsafe-template-binding-good"></span>

[正确示例](./type-and-script.md#type-no-unsafe-template-binding-good)

[分组示例](./type-and-script.md#type-no-unsafe-template-binding) · [规则参考](/rules/reference/type-no-unsafe-template-binding.md)

### `type/require-typed-emits`

要求 defineEmits 具有类型定义

<span id="type-require-typed-emits-bad"></span>

[错误示例](./type-and-script.md#type-require-typed-emits-bad)

<span id="type-require-typed-emits-good"></span>

[正确示例](./type-and-script.md#type-require-typed-emits-good)

[分组示例](./type-and-script.md#type-require-typed-emits) · [规则参考](/rules/reference/type-require-typed-emits.md)

### `type/require-typed-props`

要求 defineProps 具有类型定义

<span id="type-require-typed-props-bad"></span>

[错误示例](./type-and-script.md#type-require-typed-props-bad)

<span id="type-require-typed-props-good"></span>

[正确示例](./type-and-script.md#type-require-typed-props-good)

[分组示例](./type-and-script.md#type-require-typed-props) · [规则参考](/rules/reference/type-require-typed-props.md)

### `type/strict-boolean-expressions`

要求脚本和模板条件使用安全的布尔表达式

<span id="type-strict-boolean-expressions-bad"></span>

[错误示例](./type-and-script.md#type-strict-boolean-expressions-bad)

<span id="type-strict-boolean-expressions-good"></span>

[正确示例](./type-and-script.md#type-strict-boolean-expressions-good)

[分组示例](./type-and-script.md#type-strict-boolean-expressions) · [规则参考](/rules/reference/type-strict-boolean-expressions.md)

### `vapor/no-inline-template`

禁止已弃用的 inline-template 属性

<span id="vapor-no-inline-template-bad"></span>

[错误示例](./vapor.md#vapor-no-inline-template-bad)

<span id="vapor-no-inline-template-good"></span>

[正确示例](./vapor.md#vapor-no-inline-template-good)

[分组示例](./vapor.md#vapor-no-inline-template) · [规则参考](/rules/reference/vapor-no-inline-template.md)

### `vapor/no-vue-lifecycle-events`

禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）

<span id="vapor-no-vue-lifecycle-events-bad"></span>

[错误示例](./vapor.md#vapor-no-vue-lifecycle-events-bad)

<span id="vapor-no-vue-lifecycle-events-good"></span>

[正确示例](./vapor.md#vapor-no-vue-lifecycle-events-good)

[分组示例](./vapor.md#vapor-no-vue-lifecycle-events) · [规则参考](/rules/reference/vapor-no-vue-lifecycle-events.md)

### `vapor/prefer-static-class`

字符串字面量优先使用静态 class，而不是动态 class 绑定

<span id="vapor-prefer-static-class-bad"></span>

[错误示例](./vapor.md#vapor-prefer-static-class-bad)

<span id="vapor-prefer-static-class-good"></span>

[正确示例](./vapor.md#vapor-prefer-static-class-good)

[分组示例](./vapor.md#vapor-prefer-static-class) · [规则参考](/rules/reference/vapor-prefer-static-class.md)

### `vapor/require-vapor-attribute`

建议为 script setup 添加 vapor 属性

<span id="vapor-require-vapor-attribute-bad"></span>

[错误示例](./vapor.md#vapor-require-vapor-attribute-bad)

<span id="vapor-require-vapor-attribute-good"></span>

[正确示例](./vapor.md#vapor-require-vapor-attribute-good)

[分组示例](./vapor.md#vapor-require-vapor-attribute) · [规则参考](/rules/reference/vapor-require-vapor-attribute.md)

### `vize:croquis/cf/array-mutation`

通过索引修改数组，无法被该响应式数组跟踪。

<span id="vize-croquis-cf-array-mutation-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-array-mutation-bad)

<span id="vize-croquis-cf-array-mutation-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-array-mutation-good)

[分组示例](./cross-file.md#vize-croquis-cf-array-mutation) · [规则参考](/rules/project/vize-croquis-cf-array-mutation.md)

### `vize:croquis/cf/async-boundary`

响应式状态跨越异步边界，可能被观察为过期值。

<span id="vize-croquis-cf-async-boundary-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-async-boundary-bad)

<span id="vize-croquis-cf-async-boundary-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-async-boundary-good)

[分组示例](./cross-file.md#vize-croquis-cf-async-boundary) · [规则参考](/rules/project/vize-croquis-cf-async-boundary.md)

### `vize:croquis/cf/async-no-suspense`

异步组件在没有 Suspense 边界的情况下渲染。

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-async-no-suspense-bad)

<span id="vize-croquis-cf-async-no-suspense-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-async-no-suspense-good)

[分组示例](./cross-file.md#vize-croquis-cf-async-no-suspense) · [规则参考](/rules/project/vize-croquis-cf-async-no-suspense.md)

### `vize:croquis/cf/browser-api-ssr`

在组件可能于服务端渲染的位置使用了浏览器专用 API。

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-browser-api-ssr-bad)

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-browser-api-ssr-good)

[分组示例](./cross-file.md#vize-croquis-cf-browser-api-ssr) · [规则参考](/rules/project/vize-croquis-cf-browser-api-ssr.md)

### `vize:croquis/cf/circular-dep`

组件相互导入，形成循环。

<span id="vize-croquis-cf-circular-dep-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-circular-dep-bad)

<span id="vize-croquis-cf-circular-dep-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-circular-dep-good)

[分组示例](./cross-file.md#vize-croquis-cf-circular-dep) · [规则参考](/rules/project/vize-croquis-cf-circular-dep.md)

### `vize:croquis/cf/circular-reactive-dependency`

响应式计算相互依赖，形成循环。

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-bad)

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-circular-reactive-dependency-good)

[分组示例](./cross-file.md#vize-croquis-cf-circular-reactive-dependency) · [规则参考](/rules/project/vize-croquis-cf-circular-reactive-dependency.md)

### `vize:croquis/cf/closure-captures-reactive`

闭包捕获了响应式值，无法看到后续更新。

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-closure-captures-reactive-bad)

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-closure-captures-reactive-good)

[分组示例](./cross-file.md#vize-croquis-cf-closure-captures-reactive) · [规则参考](/rules/project/vize-croquis-cf-closure-captures-reactive.md)

### `vize:croquis/cf/composable-outside-setup`

组合式函数在 `setup` 外被调用。

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-composable-outside-setup-bad)

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-composable-outside-setup-good)

[分组示例](./cross-file.md#vize-croquis-cf-composable-outside-setup) · [规则参考](/rules/project/vize-croquis-cf-composable-outside-setup.md)

### `vize:croquis/cf/computed-side-effects`

计算 getter 写入状态或执行其他副作用。

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-computed-side-effects-bad)

<span id="vize-croquis-cf-computed-side-effects-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-computed-side-effects-good)

[分组示例](./cross-file.md#vize-croquis-cf-computed-side-effects) · [规则参考](/rules/project/vize-croquis-cf-computed-side-effects.md)

### `vize:croquis/cf/deep-import`

导入链超过项目允许的深度。

<span id="vize-croquis-cf-deep-import-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-deep-import-bad)

<span id="vize-croquis-cf-deep-import-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-deep-import-good)

[分组示例](./cross-file.md#vize-croquis-cf-deep-import) · [规则参考](/rules/project/vize-croquis-cf-deep-import.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

解构响应式对象会复制字段并丢失跟踪。

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-bad)

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity-good)

[分组示例](./cross-file.md#vize-croquis-cf-destructuring-breaks-reactivity) · [规则参考](/rules/project/vize-croquis-cf-destructuring-breaks-reactivity.md)

### `vize:croquis/cf/di-outside-setup`

`provide` 或 `inject` 在 `setup` 外被调用。

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-di-outside-setup-bad)

<span id="vize-croquis-cf-di-outside-setup-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-di-outside-setup-good)

[分组示例](./cross-file.md#vize-croquis-cf-di-outside-setup) · [规则参考](/rules/project/vize-croquis-cf-di-outside-setup.md)

### `vize:croquis/cf/dom-access-without-next-tick`

在 Vue 完成更新刷新之前读取 DOM。

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-bad)

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick-good)

[分组示例](./cross-file.md#vize-croquis-cf-dom-access-without-next-tick) · [规则参考](/rules/project/vize-croquis-cf-dom-access-without-next-tick.md)

### `vize:croquis/cf/duplicate-id`

多个组件使用同一个元素 id。

<span id="vize-croquis-cf-duplicate-id-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-duplicate-id-bad)

<span id="vize-croquis-cf-duplicate-id-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-duplicate-id-good)

[分组示例](./cross-file.md#vize-croquis-cf-duplicate-id) · [规则参考](/rules/project/vize-croquis-cf-duplicate-id.md)

### `vize:croquis/cf/event-listener-leak`

注册了事件监听器，却从未移除。

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-event-listener-leak-bad)

<span id="vize-croquis-cf-event-listener-leak-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-event-listener-leak-good)

[分组示例](./cross-file.md#vize-croquis-cf-event-listener-leak) · [规则参考](/rules/project/vize-croquis-cf-event-listener-leak.md)

### `vize:croquis/cf/event-modifier`

事件监听器使用了发出事件不支持的修饰符。

<span id="vize-croquis-cf-event-modifier-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-event-modifier-bad)

<span id="vize-croquis-cf-event-modifier-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-event-modifier-good)

[分组示例](./cross-file.md#vize-croquis-cf-event-modifier) · [规则参考](/rules/project/vize-croquis-cf-event-modifier.md)

### `vize:croquis/cf/hydration-risk`

此诊断代码归类多种响应性问题，包括将 prop 复制到 ref。它并不表示跨文件分析遍会检测所有 Date.now() 表达式。

<span id="vize-croquis-cf-hydration-risk-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-hydration-risk-bad)

<span id="vize-croquis-cf-hydration-risk-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-hydration-risk-good)

[分组示例](./cross-file.md#vize-croquis-cf-hydration-risk) · [规则参考](/rules/project/vize-croquis-cf-hydration-risk.md)

### `vize:croquis/cf/inherit-attrs-unused`

设置了 `inheritAttrs: false`，但组件从不读取属性。

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-bad)

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-inherit-attrs-unused-good)

[分组示例](./cross-file.md#vize-croquis-cf-inherit-attrs-unused) · [规则参考](/rules/project/vize-croquis-cf-inherit-attrs-unused.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` 使用普通键，而不是 `InjectionKey` symbol。

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-inject-without-symbol-bad)

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-inject-without-symbol-good)

[分组示例](./cross-file.md#vize-croquis-cf-inject-without-symbol) · [规则参考](/rules/project/vize-croquis-cf-inject-without-symbol.md)

### `vize:croquis/cf/injected-async-mutation-race`

注入值被可能发生竞态的异步任务修改。

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-bad)

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-injected-async-mutation-race-good)

[分组示例](./cross-file.md#vize-croquis-cf-injected-async-mutation-race) · [规则参考](/rules/project/vize-croquis-cf-injected-async-mutation-race.md)

### `vize:croquis/cf/lifecycle-outside-setup`

生命周期钩子在 `setup` 外注册。

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-bad)

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup-good)

[分组示例](./cross-file.md#vize-croquis-cf-lifecycle-outside-setup) · [规则参考](/rules/project/vize-croquis-cf-lifecycle-outside-setup.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

生命周期钩子启动任务，却从不清理。

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-bad)

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup-good)

[分组示例](./cross-file.md#vize-croquis-cf-lifecycle-without-cleanup) · [规则参考](/rules/project/vize-croquis-cf-lifecycle-without-cleanup.md)

### `vize:croquis/cf/missing-required-prop`

未传入必需的 prop。

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-missing-required-prop-bad)

<span id="vize-croquis-cf-missing-required-prop-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-missing-required-prop-good)

[分组示例](./cross-file.md#vize-croquis-cf-missing-required-prop) · [规则参考](/rules/project/vize-croquis-cf-missing-required-prop.md)

### `vize:croquis/cf/missing-suspense`

异步依赖在 Suspense 边界之外使用。

<span id="vize-croquis-cf-missing-suspense-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-missing-suspense-bad)

<span id="vize-croquis-cf-missing-suspense-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-missing-suspense-good)

[分组示例](./cross-file.md#vize-croquis-cf-missing-suspense) · [规则参考](/rules/project/vize-croquis-cf-missing-suspense.md)

### `vize:croquis/cf/module-scope-reactive`

响应式状态在模块作用域创建，被所有调用者共享。

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-module-scope-reactive-bad)

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-module-scope-reactive-good)

[分组示例](./cross-file.md#vize-croquis-cf-module-scope-reactive) · [规则参考](/rules/project/vize-croquis-cf-module-scope-reactive.md)

### `vize:croquis/cf/multi-root-attrs`

多根节点组件接收了属性，却没有放置目标。

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-multi-root-attrs-bad)

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-multi-root-attrs-good)

[分组示例](./cross-file.md#vize-croquis-cf-multi-root-attrs) · [规则参考](/rules/project/vize-croquis-cf-multi-root-attrs.md)

### `vize:croquis/cf/mutated-after-escape`

响应式对象逸出其所有者后被修改。

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-mutated-after-escape-bad)

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-mutated-after-escape-good)

[分组示例](./cross-file.md#vize-croquis-cf-mutated-after-escape) · [规则参考](/rules/project/vize-croquis-cf-mutated-after-escape.md)

### `vize:croquis/cf/non-reactive-provide`

提供的值不具响应性，后代无法看到更新。

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-non-reactive-provide-bad)

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-non-reactive-provide-good)

[分组示例](./cross-file.md#vize-croquis-cf-non-reactive-provide) · [规则参考](/rules/project/vize-croquis-cf-non-reactive-provide.md)

### `vize:croquis/cf/non-unique-id`

循环内的元素 id 对每个列表项并不唯一。

<span id="vize-croquis-cf-non-unique-id-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-non-unique-id-bad)

<span id="vize-croquis-cf-non-unique-id-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-non-unique-id-good)

[分组示例](./cross-file.md#vize-croquis-cf-non-unique-id) · [规则参考](/rules/project/vize-croquis-cf-non-unique-id.md)

### `vize:croquis/cf/object-identity-comparison`

响应式对象按身份比较，解包后身份会改变。

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-object-identity-comparison-bad)

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-object-identity-comparison-good)

[分组示例](./cross-file.md#vize-croquis-cf-object-identity-comparison) · [规则参考](/rules/project/vize-croquis-cf-object-identity-comparison.md)

### `vize:croquis/cf/pinia-getter`

Pinia getter 未通过 `storeToRefs` 读取，无法保持响应性。

<span id="vize-croquis-cf-pinia-getter-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-pinia-getter-bad)

<span id="vize-croquis-cf-pinia-getter-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-pinia-getter-good)

[分组示例](./cross-file.md#vize-croquis-cf-pinia-getter) · [规则参考](/rules/project/vize-croquis-cf-pinia-getter.md)

### `vize:croquis/cf/prop-type-mismatch`

传入的 prop 值与声明类型不匹配。

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-prop-type-mismatch-bad)

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-prop-type-mismatch-good)

[分组示例](./cross-file.md#vize-croquis-cf-prop-type-mismatch) · [规则参考](/rules/project/vize-croquis-cf-prop-type-mismatch.md)

### `vize:croquis/cf/provide-inject-type`

提供值与其 inject 类型不一致。

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-provide-inject-type-bad)

<span id="vize-croquis-cf-provide-inject-type-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-provide-inject-type-good)

[分组示例](./cross-file.md#vize-croquis-cf-provide-inject-type) · [规则参考](/rules/project/vize-croquis-cf-provide-inject-type.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` 使用普通键，而不是 `InjectionKey` symbol。

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-provide-without-symbol-bad)

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-provide-without-symbol-good)

[分组示例](./cross-file.md#vize-croquis-cf-provide-without-symbol) · [规则参考](/rules/project/vize-croquis-cf-provide-without-symbol.md)

### `vize:croquis/cf/reactive-export`

响应式状态从模块导出。

<span id="vize-croquis-cf-reactive-export-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-reactive-export-bad)

<span id="vize-croquis-cf-reactive-export-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-reactive-export-good)

[分组示例](./cross-file.md#vize-croquis-cf-reactive-export) · [规则参考](/rules/project/vize-croquis-cf-reactive-export.md)

### `vize:croquis/cf/reactivity-outside-setup`

响应式 API 在 `setup` 外被调用。

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-bad)

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-reactivity-outside-setup-good)

[分组示例](./cross-file.md#vize-croquis-cf-reactivity-outside-setup) · [规则参考](/rules/project/vize-croquis-cf-reactivity-outside-setup.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

重新赋值响应式绑定，将其替换为普通值。

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-bad)

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity-good)

[分组示例](./cross-file.md#vize-croquis-cf-reassignment-breaks-reactivity) · [规则参考](/rules/project/vize-croquis-cf-reassignment-breaks-reactivity.md)

### `vize:croquis/cf/reference-escapes-scope`

响应式引用逸出拥有其生命周期的作用域。

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-reference-escapes-scope-bad)

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-reference-escapes-scope-good)

[分组示例](./cross-file.md#vize-croquis-cf-reference-escapes-scope) · [规则参考](/rules/project/vize-croquis-cf-reference-escapes-scope.md)

### `vize:croquis/cf/setup-context-violation`

setup 上下文被以 Vue 不允许的方式使用。

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-setup-context-violation-bad)

<span id="vize-croquis-cf-setup-context-violation-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-setup-context-violation-good)

[分组示例](./cross-file.md#vize-croquis-cf-setup-context-violation) · [规则参考](/rules/project/vize-croquis-cf-setup-context-violation.md)

### `vize:croquis/cf/shallow-deep-access`

读取 `shallowReactive` 或 `shallowRef` 的深层属性，却假定它受到跟踪。

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-shallow-deep-access-bad)

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-shallow-deep-access-good)

[分组示例](./cross-file.md#vize-croquis-cf-shallow-deep-access) · [规则参考](/rules/project/vize-croquis-cf-shallow-deep-access.md)

### `vize:croquis/cf/spread-breaks-reactivity`

展开响应式对象会复制值并丢失跟踪。

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-bad)

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity-good)

[分组示例](./cross-file.md#vize-croquis-cf-spread-breaks-reactivity) · [规则参考](/rules/project/vize-croquis-cf-spread-breaks-reactivity.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` 没有后备内容。

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-suspense-no-fallback-bad)

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-suspense-no-fallback-good)

[分组示例](./cross-file.md#vize-croquis-cf-suspense-no-fallback) · [规则参考](/rules/project/vize-croquis-cf-suspense-no-fallback.md)

### `vize:croquis/cf/template-ref-timing`

模板 ref 在组件挂载之前被读取。

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-template-ref-timing-bad)

<span id="vize-croquis-cf-template-ref-timing-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-template-ref-timing-good)

[分组示例](./cross-file.md#vize-croquis-cf-template-ref-timing) · [规则参考](/rules/project/vize-croquis-cf-template-ref-timing.md)

### `vize:croquis/cf/toraw-mutation`

使用 `toRaw` 后修改了原始对象。

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-toraw-mutation-bad)

<span id="vize-croquis-cf-toraw-mutation-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-toraw-mutation-good)

[分组示例](./cross-file.md#vize-croquis-cf-toraw-mutation) · [规则参考](/rules/project/vize-croquis-cf-toraw-mutation.md)

### `vize:croquis/cf/uncaught-error`

组件可能抛错，却没有错误边界捕获。

<span id="vize-croquis-cf-uncaught-error-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-uncaught-error-bad)

<span id="vize-croquis-cf-uncaught-error-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-uncaught-error-good)

[分组示例](./cross-file.md#vize-croquis-cf-uncaught-error) · [规则参考](/rules/project/vize-croquis-cf-uncaught-error.md)

### `vize:croquis/cf/undeclared-emit`

组件发出了未声明的事件。

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-undeclared-emit-bad)

<span id="vize-croquis-cf-undeclared-emit-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-undeclared-emit-good)

[分组示例](./cross-file.md#vize-croquis-cf-undeclared-emit) · [规则参考](/rules/project/vize-croquis-cf-undeclared-emit.md)

### `vize:croquis/cf/undeclared-prop`

父组件传入了子组件未声明的 prop。

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-undeclared-prop-bad)

<span id="vize-croquis-cf-undeclared-prop-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-undeclared-prop-good)

[分组示例](./cross-file.md#vize-croquis-cf-undeclared-prop) · [规则参考](/rules/project/vize-croquis-cf-undeclared-prop.md)

### `vize:croquis/cf/undefined-slot`

父组件填充了子组件未暴露的插槽。

<span id="vize-croquis-cf-undefined-slot-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-undefined-slot-bad)

<span id="vize-croquis-cf-undefined-slot-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-undefined-slot-good)

[分组示例](./cross-file.md#vize-croquis-cf-undefined-slot) · [规则参考](/rules/project/vize-croquis-cf-undefined-slot.md)

### `vize:croquis/cf/unhandled-event`

子组件发出的事件没有父组件处理。

<span id="vize-croquis-cf-unhandled-event-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unhandled-event-bad)

<span id="vize-croquis-cf-unhandled-event-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unhandled-event-good)

[分组示例](./cross-file.md#vize-croquis-cf-unhandled-event) · [规则参考](/rules/project/vize-croquis-cf-unhandled-event.md)

### `vize:croquis/cf/unmatched-inject`

`inject` 指定的键没有任何祖先提供。

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unmatched-inject-bad)

<span id="vize-croquis-cf-unmatched-inject-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unmatched-inject-good)

[分组示例](./cross-file.md#vize-croquis-cf-unmatched-inject) · [规则参考](/rules/project/vize-croquis-cf-unmatched-inject.md)

### `vize:croquis/cf/unmatched-listener`

父组件监听了子组件不发出的事件。

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unmatched-listener-bad)

<span id="vize-croquis-cf-unmatched-listener-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unmatched-listener-good)

[分组示例](./cross-file.md#vize-croquis-cf-unmatched-listener) · [规则参考](/rules/project/vize-croquis-cf-unmatched-listener.md)

### `vize:croquis/cf/unregistered-component`

模板使用了未注册或导入的组件。

<span id="vize-croquis-cf-unregistered-component-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unregistered-component-bad)

<span id="vize-croquis-cf-unregistered-component-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unregistered-component-good)

[分组示例](./cross-file.md#vize-croquis-cf-unregistered-component) · [规则参考](/rules/project/vize-croquis-cf-unregistered-component.md)

### `vize:croquis/cf/unresolved-import`

导入无法解析为模块。

<span id="vize-croquis-cf-unresolved-import-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unresolved-import-bad)

<span id="vize-croquis-cf-unresolved-import-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unresolved-import-good)

[分组示例](./cross-file.md#vize-croquis-cf-unresolved-import) · [规则参考](/rules/project/vize-croquis-cf-unresolved-import.md)

### `vize:croquis/cf/unused-attrs`

透传属性传给了未使用它们的多根节点组件。

<span id="vize-croquis-cf-unused-attrs-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unused-attrs-bad)

<span id="vize-croquis-cf-unused-attrs-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unused-attrs-good)

[分组示例](./cross-file.md#vize-croquis-cf-unused-attrs) · [规则参考](/rules/project/vize-croquis-cf-unused-attrs.md)

### `vize:croquis/cf/unused-emit`

已声明的 emit 从未使用。

<span id="vize-croquis-cf-unused-emit-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unused-emit-bad)

<span id="vize-croquis-cf-unused-emit-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unused-emit-good)

[分组示例](./cross-file.md#vize-croquis-cf-unused-emit) · [规则参考](/rules/project/vize-croquis-cf-unused-emit.md)

### `vize:croquis/cf/unused-provide`

提供的键从未被注入。

<span id="vize-croquis-cf-unused-provide-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-unused-provide-bad)

<span id="vize-croquis-cf-unused-provide-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-unused-provide-good)

[分组示例](./cross-file.md#vize-croquis-cf-unused-provide) · [规则参考](/rules/project/vize-croquis-cf-unused-provide.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

将响应式值读取到局部变量后丢失后续更新。

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-bad)

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity-good)

[分组示例](./cross-file.md#vize-croquis-cf-value-extraction-breaks-reactivity) · [规则参考](/rules/project/vize-croquis-cf-value-extraction-breaks-reactivity.md)

### `vize:croquis/cf/watch-can-be-computed`

侦听器只将值复制到状态，可以改为计算属性。

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-watch-can-be-computed-bad)

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-watch-can-be-computed-good)

[分组示例](./cross-file.md#vize-croquis-cf-watch-can-be-computed) · [规则参考](/rules/project/vize-croquis-cf-watch-can-be-computed.md)

### `vize:croquis/cf/watcheffect-async`

`watchEffect` 启动异步任务，却无法清理上一轮执行。

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-watcheffect-async-bad)

<span id="vize-croquis-cf-watcheffect-async-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-watcheffect-async-good)

[分组示例](./cross-file.md#vize-croquis-cf-watcheffect-async) · [规则参考](/rules/project/vize-croquis-cf-watcheffect-async.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` 或 `watchEffect` 在 `setup` 外被调用。

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

[错误示例](./cross-file.md#vize-croquis-cf-watcher-outside-setup-bad)

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

[正确示例](./cross-file.md#vize-croquis-cf-watcher-outside-setup-good)

[分组示例](./cross-file.md#vize-croquis-cf-watcher-outside-setup) · [规则参考](/rules/project/vize-croquis-cf-watcher-outside-setup.md)

### `vue/a11y-img-alt`

要求图片提供 alt 属性，以支持无障碍访问

<span id="vue-a11y-img-alt-bad"></span>

[错误示例](./vue.md#vue-a11y-img-alt-bad)

<span id="vue-a11y-img-alt-good"></span>

[正确示例](./vue.md#vue-a11y-img-alt-good)

[分组示例](./vue.md#vue-a11y-img-alt) · [规则参考](/rules/reference/vue-a11y-img-alt.md)

### `vue/attribute-hyphenation`

规范自定义组件的属性命名风格

<span id="vue-attribute-hyphenation-bad"></span>

[错误示例](./vue.md#vue-attribute-hyphenation-bad)

<span id="vue-attribute-hyphenation-good"></span>

[正确示例](./vue.md#vue-attribute-hyphenation-good)

[分组示例](./vue.md#vue-attribute-hyphenation) · [规则参考](/rules/reference/vue-attribute-hyphenation.md)

### `vue/attribute-order`

要求属性保持一致的排列顺序

<span id="vue-attribute-order-bad"></span>

[错误示例](./vue.md#vue-attribute-order-bad)

<span id="vue-attribute-order-good"></span>

[正确示例](./vue.md#vue-attribute-order-good)

[分组示例](./vue.md#vue-attribute-order) · [规则参考](/rules/reference/vue-attribute-order.md)

### `vue/component-definition-name-casing`

要求组件定义名称使用 PascalCase 或 kebab-case

<span id="vue-component-definition-name-casing-bad"></span>

[错误示例](./vue.md#vue-component-definition-name-casing-bad)

<span id="vue-component-definition-name-casing-good"></span>

[正确示例](./vue.md#vue-component-definition-name-casing-good)

[分组示例](./vue.md#vue-component-definition-name-casing) · [规则参考](/rules/reference/vue-component-definition-name-casing.md)

### `vue/component-name-in-template-casing`

要求模板中的组件名称使用指定的大小写风格

<span id="vue-component-name-in-template-casing-bad"></span>

[错误示例](./vue.md#vue-component-name-in-template-casing-bad)

<span id="vue-component-name-in-template-casing-good"></span>

[正确示例](./vue.md#vue-component-name-in-template-casing-good)

[分组示例](./vue.md#vue-component-name-in-template-casing) · [规则参考](/rules/reference/vue-component-name-in-template-casing.md)

### `vue/cross-file-attrs-fallthrough`

父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

[错误示例](./cross-file.md#vue-cross-file-attrs-fallthrough-bad)

<span id="vue-cross-file-attrs-fallthrough-good"></span>

[正确示例](./cross-file.md#vue-cross-file-attrs-fallthrough-good)

[分组示例](./cross-file.md#vue-cross-file-attrs-fallthrough) · [规则参考](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `vue/html-button-has-type`

要求 button 元素显式指定有效的 type

<span id="vue-html-button-has-type-bad"></span>

[错误示例](./vue.md#vue-html-button-has-type-bad)

<span id="vue-html-button-has-type-good"></span>

[正确示例](./vue.md#vue-html-button-has-type-good)

[分组示例](./vue.md#vue-html-button-has-type) · [规则参考](/rules/reference/vue-html-button-has-type.md)

### `vue/html-quotes`

规范 HTML 属性的引号风格

<span id="vue-html-quotes-bad"></span>

[错误示例](./vue.md#vue-html-quotes-bad)

<span id="vue-html-quotes-good"></span>

[正确示例](./vue.md#vue-html-quotes-good)

[分组示例](./vue.md#vue-html-quotes) · [规则参考](/rules/reference/vue-html-quotes.md)

### `vue/html-self-closing`

规范自闭合标签风格

<span id="vue-html-self-closing-bad"></span>

[错误示例](./vue.md#vue-html-self-closing-bad)

<span id="vue-html-self-closing-good"></span>

[正确示例](./vue.md#vue-html-self-closing-good)

[分组示例](./vue.md#vue-html-self-closing) · [规则参考](/rules/reference/vue-html-self-closing.md)

### `vue/max-template-complexity`

限制组件自身模板的复杂度（圈复杂度和认知复杂度）

<span id="vue-max-template-complexity-bad"></span>

[错误示例](./vue.md#vue-max-template-complexity-bad)

<span id="vue-max-template-complexity-good"></span>

[正确示例](./vue.md#vue-max-template-complexity-good)

[分组示例](./vue.md#vue-max-template-complexity) · [规则参考](/rules/reference/vue-max-template-complexity.md)

### `vue/multi-word-component-names`

要求组件名称由多个单词组成

<span id="vue-multi-word-component-names-bad"></span>

[错误示例](./vue.md#vue-multi-word-component-names-bad)

<span id="vue-multi-word-component-names-good"></span>

[正确示例](./vue.md#vue-multi-word-component-names-good)

[分组示例](./vue.md#vue-multi-word-component-names) · [规则参考](/rules/reference/vue-multi-word-component-names.md)

### `vue/mustache-interpolation-spacing`

要求双花括号插值内部的空格保持一致

<span id="vue-mustache-interpolation-spacing-bad"></span>

[错误示例](./vue.md#vue-mustache-interpolation-spacing-bad)

<span id="vue-mustache-interpolation-spacing-good"></span>

[正确示例](./vue.md#vue-mustache-interpolation-spacing-good)

[分组示例](./vue.md#vue-mustache-interpolation-spacing) · [规则参考](/rules/reference/vue-mustache-interpolation-spacing.md)

### `vue/no-array-index-key`

禁止直接将 v-for 的索引变量用作 :key

<span id="vue-no-array-index-key-bad"></span>

[错误示例](./vue.md#vue-no-array-index-key-bad)

<span id="vue-no-array-index-key-good"></span>

[正确示例](./vue.md#vue-no-array-index-key-good)

[分组示例](./vue.md#vue-no-array-index-key) · [规则参考](/rules/reference/vue-no-array-index-key.md)

### `vue/no-bare-strings-in-template`

禁止在模板中直接写入应当国际化的用户可读文本

<span id="vue-no-bare-strings-in-template-bad"></span>

[错误示例](./vue.md#vue-no-bare-strings-in-template-bad)

<span id="vue-no-bare-strings-in-template-good"></span>

[正确示例](./vue.md#vue-no-bare-strings-in-template-good)

[分组示例](./vue.md#vue-no-bare-strings-in-template) · [规则参考](/rules/reference/vue-no-bare-strings-in-template.md)

### `vue/no-boolean-attr-value`

禁止为 HTML 布尔属性显式指定值

<span id="vue-no-boolean-attr-value-bad"></span>

[错误示例](./vue.md#vue-no-boolean-attr-value-bad)

<span id="vue-no-boolean-attr-value-good"></span>

[正确示例](./vue.md#vue-no-boolean-attr-value-good)

[分组示例](./vue.md#vue-no-boolean-attr-value) · [规则参考](/rules/reference/vue-no-boolean-attr-value.md)

### `vue/no-child-content`

禁止在使用 v-html 或 v-text 时提供子内容

<span id="vue-no-child-content-bad"></span>

[错误示例](./vue.md#vue-no-child-content-bad)

<span id="vue-no-child-content-good"></span>

[正确示例](./vue.md#vue-no-child-content-good)

[分组示例](./vue.md#vue-no-child-content) · [规则参考](/rules/reference/vue-no-child-content.md)

### `vue/no-deprecated-filter`

禁止使用已弃用的 Vue 2 管道运算符过滤器语法

<span id="vue-no-deprecated-filter-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-filter-bad)

<span id="vue-no-deprecated-filter-good"></span>

[正确示例](./vue.md#vue-no-deprecated-filter-good)

[分组示例](./vue.md#vue-no-deprecated-filter) · [规则参考](/rules/reference/vue-no-deprecated-filter.md)

### `vue/no-deprecated-functional-template`

禁止在 SFC 的 `<template>` 上使用 `functional` 属性

<span id="vue-no-deprecated-functional-template-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-functional-template-bad)

<span id="vue-no-deprecated-functional-template-good"></span>

[正确示例](./vue.md#vue-no-deprecated-functional-template-good)

[分组示例](./vue.md#vue-no-deprecated-functional-template) · [规则参考](/rules/reference/vue-no-deprecated-functional-template.md)

### `vue/no-deprecated-html-element-is`

禁止在原生 HTML 元素上使用已弃用的 `is` 属性

<span id="vue-no-deprecated-html-element-is-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-html-element-is-bad)

<span id="vue-no-deprecated-html-element-is-good"></span>

[正确示例](./vue.md#vue-no-deprecated-html-element-is-good)

[分组示例](./vue.md#vue-no-deprecated-html-element-is) · [规则参考](/rules/reference/vue-no-deprecated-html-element-is.md)

### `vue/no-deprecated-inline-template`

禁止使用已弃用的 `inline-template` 属性

<span id="vue-no-deprecated-inline-template-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-inline-template-bad)

<span id="vue-no-deprecated-inline-template-good"></span>

[正确示例](./vue.md#vue-no-deprecated-inline-template-good)

[分组示例](./vue.md#vue-no-deprecated-inline-template) · [规则参考](/rules/reference/vue-no-deprecated-inline-template.md)

### `vue/no-deprecated-router-link-tag-prop`

禁止在 &lt;router-link&gt; 上使用 `tag` prop

<span id="vue-no-deprecated-router-link-tag-prop-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-router-link-tag-prop-bad)

<span id="vue-no-deprecated-router-link-tag-prop-good"></span>

[正确示例](./vue.md#vue-no-deprecated-router-link-tag-prop-good)

[分组示例](./vue.md#vue-no-deprecated-router-link-tag-prop) · [规则参考](/rules/reference/vue-no-deprecated-router-link-tag-prop.md)

### `vue/no-deprecated-scope-attribute`

禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性

<span id="vue-no-deprecated-scope-attribute-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-scope-attribute-bad)

<span id="vue-no-deprecated-scope-attribute-good"></span>

[正确示例](./vue.md#vue-no-deprecated-scope-attribute-good)

[分组示例](./vue.md#vue-no-deprecated-scope-attribute) · [规则参考](/rules/reference/vue-no-deprecated-scope-attribute.md)

### `vue/no-deprecated-slot-attribute`

禁止使用已弃用的 `slot` 属性

<span id="vue-no-deprecated-slot-attribute-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-slot-attribute-bad)

<span id="vue-no-deprecated-slot-attribute-good"></span>

[正确示例](./vue.md#vue-no-deprecated-slot-attribute-good)

[分组示例](./vue.md#vue-no-deprecated-slot-attribute) · [规则参考](/rules/reference/vue-no-deprecated-slot-attribute.md)

### `vue/no-deprecated-slot-scope-attribute`

禁止使用已弃用的 `slot-scope` 属性

<span id="vue-no-deprecated-slot-scope-attribute-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-slot-scope-attribute-bad)

<span id="vue-no-deprecated-slot-scope-attribute-good"></span>

[正确示例](./vue.md#vue-no-deprecated-slot-scope-attribute-good)

[分组示例](./vue.md#vue-no-deprecated-slot-scope-attribute) · [规则参考](/rules/reference/vue-no-deprecated-slot-scope-attribute.md)

### `vue/no-deprecated-v-bind-sync`

禁止在 `v-bind` 上使用已弃用的 `.sync` 修饰符

<span id="vue-no-deprecated-v-bind-sync-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-v-bind-sync-bad)

<span id="vue-no-deprecated-v-bind-sync-good"></span>

[正确示例](./vue.md#vue-no-deprecated-v-bind-sync-good)

[分组示例](./vue.md#vue-no-deprecated-v-bind-sync) · [规则参考](/rules/reference/vue-no-deprecated-v-bind-sync.md)

### `vue/no-deprecated-v-is`

禁止使用已废弃的 `v-is` 指令

<span id="vue-no-deprecated-v-is-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-v-is-bad)

<span id="vue-no-deprecated-v-is-good"></span>

[正确示例](./vue.md#vue-no-deprecated-v-is-good)

[分组示例](./vue.md#vue-no-deprecated-v-is) · [规则参考](/rules/reference/vue-no-deprecated-v-is.md)

### `vue/no-deprecated-v-on-native-modifier`

禁止在 `v-on` 上使用已弃用的 `.native` 修饰符

<span id="vue-no-deprecated-v-on-native-modifier-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-v-on-native-modifier-bad)

<span id="vue-no-deprecated-v-on-native-modifier-good"></span>

[正确示例](./vue.md#vue-no-deprecated-v-on-native-modifier-good)

[分组示例](./vue.md#vue-no-deprecated-v-on-native-modifier) · [规则参考](/rules/reference/vue-no-deprecated-v-on-native-modifier.md)

### `vue/no-deprecated-v-on-number-modifiers`

禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符

<span id="vue-no-deprecated-v-on-number-modifiers-bad"></span>

[错误示例](./vue.md#vue-no-deprecated-v-on-number-modifiers-bad)

<span id="vue-no-deprecated-v-on-number-modifiers-good"></span>

[正确示例](./vue.md#vue-no-deprecated-v-on-number-modifiers-good)

[分组示例](./vue.md#vue-no-deprecated-v-on-number-modifiers) · [规则参考](/rules/reference/vue-no-deprecated-v-on-number-modifiers.md)

### `vue/no-dupe-v-else-if`

禁止在 `v-if` / `v-else-if` 链中使用重复条件

<span id="vue-no-dupe-v-else-if-bad"></span>

[错误示例](./vue.md#vue-no-dupe-v-else-if-bad)

<span id="vue-no-dupe-v-else-if-good"></span>

[正确示例](./vue.md#vue-no-dupe-v-else-if-good)

[分组示例](./vue.md#vue-no-dupe-v-else-if) · [规则参考](/rules/reference/vue-no-dupe-v-else-if.md)

### `vue/no-duplicate-attributes`

禁止在同一元素上重复声明属性

<span id="vue-no-duplicate-attributes-bad"></span>

[错误示例](./vue.md#vue-no-duplicate-attributes-bad)

<span id="vue-no-duplicate-attributes-good"></span>

[正确示例](./vue.md#vue-no-duplicate-attributes-good)

[分组示例](./vue.md#vue-no-duplicate-attributes) · [规则参考](/rules/reference/vue-no-duplicate-attributes.md)

### `vue/no-empty-component-block`

禁止空的 SFC 块

<span id="vue-no-empty-component-block-bad"></span>

[错误示例](./vue.md#vue-no-empty-component-block-bad)

<span id="vue-no-empty-component-block-good"></span>

[正确示例](./vue.md#vue-no-empty-component-block-good)

[分组示例](./vue.md#vue-no-empty-component-block) · [规则参考](/rules/reference/vue-no-empty-component-block.md)

### `vue/no-inline-style`

不建议使用内联 style 属性

<span id="vue-no-inline-style-bad"></span>

[错误示例](./vue.md#vue-no-inline-style-bad)

<span id="vue-no-inline-style-good"></span>

[正确示例](./vue.md#vue-no-inline-style-good)

[分组示例](./vue.md#vue-no-inline-style) · [规则参考](/rules/reference/vue-no-inline-style.md)

### `vue/no-invalid-html-attribute`

禁止 HTML 属性使用无效的静态值

<span id="vue-no-invalid-html-attribute-bad"></span>

[错误示例](./vue.md#vue-no-invalid-html-attribute-bad)

<span id="vue-no-invalid-html-attribute-good"></span>

[正确示例](./vue.md#vue-no-invalid-html-attribute-good)

[分组示例](./vue.md#vue-no-invalid-html-attribute) · [规则参考](/rules/reference/vue-no-invalid-html-attribute.md)

### `vue/no-lone-template`

禁止不必要的 `<template>` 元素

<span id="vue-no-lone-template-bad"></span>

[错误示例](./vue.md#vue-no-lone-template-bad)

<span id="vue-no-lone-template-good"></span>

[正确示例](./vue.md#vue-no-lone-template-good)

[分组示例](./vue.md#vue-no-lone-template) · [规则参考](/rules/reference/vue-no-lone-template.md)

### `vue/no-multi-spaces`

禁止连续多个空格

<span id="vue-no-multi-spaces-bad"></span>

[错误示例](./vue.md#vue-no-multi-spaces-bad)

<span id="vue-no-multi-spaces-good"></span>

[正确示例](./vue.md#vue-no-multi-spaces-good)

[分组示例](./vue.md#vue-no-multi-spaces) · [规则参考](/rules/reference/vue-no-multi-spaces.md)

### `vue/no-multiple-objects-in-class`

禁止在 :class 数组绑定中使用多个对象字面量

<span id="vue-no-multiple-objects-in-class-bad"></span>

[错误示例](./vue.md#vue-no-multiple-objects-in-class-bad)

<span id="vue-no-multiple-objects-in-class-good"></span>

[正确示例](./vue.md#vue-no-multiple-objects-in-class-good)

[分组示例](./vue.md#vue-no-multiple-objects-in-class) · [规则参考](/rules/reference/vue-no-multiple-objects-in-class.md)

### `vue/no-multiple-template-root`

禁止模板包含多个根节点

<span id="vue-no-multiple-template-root-bad"></span>

[错误示例](./vue.md#vue-no-multiple-template-root-bad)

<span id="vue-no-multiple-template-root-good"></span>

[正确示例](./vue.md#vue-no-multiple-template-root-good)

[分组示例](./vue.md#vue-no-multiple-template-root) · [规则参考](/rules/reference/vue-no-multiple-template-root.md)

### `vue/no-mutating-props`

禁止修改组件 props

<span id="vue-no-mutating-props-bad"></span>

[错误示例](./vue.md#vue-no-mutating-props-bad)

<span id="vue-no-mutating-props-good"></span>

[正确示例](./vue.md#vue-no-mutating-props-good)

[分组示例](./vue.md#vue-no-mutating-props) · [规则参考](/rules/reference/vue-no-mutating-props.md)

### `vue/no-negated-v-if-condition`

禁止在含有 v-else 的条件链中使用否定的 v-if 条件

<span id="vue-no-negated-v-if-condition-bad"></span>

[错误示例](./vue.md#vue-no-negated-v-if-condition-bad)

<span id="vue-no-negated-v-if-condition-good"></span>

[正确示例](./vue.md#vue-no-negated-v-if-condition-good)

[分组示例](./vue.md#vue-no-negated-v-if-condition) · [规则参考](/rules/reference/vue-no-negated-v-if-condition.md)

### `vue/no-non-component-keep-alive-child`

禁止在 `<KeepAlive>` 的直属子层使用普通元素包装

<span id="vue-no-non-component-keep-alive-child-bad"></span>

[错误示例](./vue.md#vue-no-non-component-keep-alive-child-bad)

<span id="vue-no-non-component-keep-alive-child-good"></span>

[正确示例](./vue.md#vue-no-non-component-keep-alive-child-good)

[分组示例](./vue.md#vue-no-non-component-keep-alive-child) · [规则参考](/rules/reference/vue-no-non-component-keep-alive-child.md)

### `vue/no-preprocessor-lang`

不建议使用 CSS 预处理器，优先使用现代 CSS

<span id="vue-no-preprocessor-lang-bad"></span>

[错误示例](./vue.md#vue-no-preprocessor-lang-bad)

<span id="vue-no-preprocessor-lang-good"></span>

[正确示例](./vue.md#vue-no-preprocessor-lang-good)

[分组示例](./vue.md#vue-no-preprocessor-lang) · [规则参考](/rules/reference/vue-no-preprocessor-lang.md)

### `vue/no-reserved-component-names`

禁止使用保留名称作为组件名称

<span id="vue-no-reserved-component-names-bad"></span>

[错误示例](./vue.md#vue-no-reserved-component-names-bad)

<span id="vue-no-reserved-component-names-good"></span>

[正确示例](./vue.md#vue-no-reserved-component-names-good)

[分组示例](./vue.md#vue-no-reserved-component-names) · [规则参考](/rules/reference/vue-no-reserved-component-names.md)

### `vue/no-root-v-if`

禁止在模板的唯一根元素上使用 v-if

<span id="vue-no-root-v-if-bad"></span>

[错误示例](./vue.md#vue-no-root-v-if-bad)

<span id="vue-no-root-v-if-good"></span>

[正确示例](./vue.md#vue-no-root-v-if-good)

[分组示例](./vue.md#vue-no-root-v-if) · [规则参考](/rules/reference/vue-no-root-v-if.md)

### `vue/no-script-non-standard-lang`

不建议使用非标准的 script lang 值

<span id="vue-no-script-non-standard-lang-bad"></span>

[错误示例](./vue.md#vue-no-script-non-standard-lang-bad)

<span id="vue-no-script-non-standard-lang-good"></span>

[正确示例](./vue.md#vue-no-script-non-standard-lang-good)

[分组示例](./vue.md#vue-no-script-non-standard-lang) · [规则参考](/rules/reference/vue-no-script-non-standard-lang.md)

### `vue/no-src-attribute`

不建议在 SFC 块上使用 src 属性

<span id="vue-no-src-attribute-bad"></span>

[错误示例](./vue.md#vue-no-src-attribute-bad)

<span id="vue-no-src-attribute-good"></span>

[正确示例](./vue.md#vue-no-src-attribute-good)

[分组示例](./vue.md#vue-no-src-attribute) · [规则参考](/rules/reference/vue-no-src-attribute.md)

### `vue/no-static-inline-styles`

禁止静态内联 style 属性

<span id="vue-no-static-inline-styles-bad"></span>

[错误示例](./vue.md#vue-no-static-inline-styles-bad)

<span id="vue-no-static-inline-styles-good"></span>

[正确示例](./vue.md#vue-no-static-inline-styles-good)

[分组示例](./vue.md#vue-no-static-inline-styles) · [规则参考](/rules/reference/vue-no-static-inline-styles.md)

### `vue/no-template-key`

禁止在 `<template>` 上使用 `key` 属性

<span id="vue-no-template-key-bad"></span>

[错误示例](./vue.md#vue-no-template-key-bad)

<span id="vue-no-template-key-good"></span>

[正确示例](./vue.md#vue-no-template-key-good)

[分组示例](./vue.md#vue-no-template-key) · [规则参考](/rules/reference/vue-no-template-key.md)

### `vue/no-template-lang`

不建议在 template 块上使用 lang 属性

<span id="vue-no-template-lang-bad"></span>

[错误示例](./vue.md#vue-no-template-lang-bad)

<span id="vue-no-template-lang-good"></span>

[正确示例](./vue.md#vue-no-template-lang-good)

[分组示例](./vue.md#vue-no-template-lang) · [规则参考](/rules/reference/vue-no-template-lang.md)

### `vue/no-template-shadow`

禁止使用遮蔽外层作用域变量的变量名称

<span id="vue-no-template-shadow-bad"></span>

[错误示例](./vue.md#vue-no-template-shadow-bad)

<span id="vue-no-template-shadow-good"></span>

[正确示例](./vue.md#vue-no-template-shadow-good)

[分组示例](./vue.md#vue-no-template-shadow) · [规则参考](/rules/reference/vue-no-template-shadow.md)

### `vue/no-template-target-blank`

禁止使用 target="_blank" 而不设置 rel="noopener noreferrer"

<span id="vue-no-template-target-blank-bad"></span>

[错误示例](./vue.md#vue-no-template-target-blank-bad)

<span id="vue-no-template-target-blank-good"></span>

[正确示例](./vue.md#vue-no-template-target-blank-good)

[分组示例](./vue.md#vue-no-template-target-blank) · [规则参考](/rules/reference/vue-no-template-target-blank.md)

### `vue/no-textarea-mustache`

禁止在 `<textarea>` 内使用双花括号插值

<span id="vue-no-textarea-mustache-bad"></span>

[错误示例](./vue.md#vue-no-textarea-mustache-bad)

<span id="vue-no-textarea-mustache-good"></span>

[正确示例](./vue.md#vue-no-textarea-mustache-good)

[分组示例](./vue.md#vue-no-textarea-mustache) · [规则参考](/rules/reference/vue-no-textarea-mustache.md)

### `vue/no-undefined-refs`

禁止在模板中引用未定义的变量

<span id="vue-no-undefined-refs-bad"></span>

[错误示例](./vue.md#vue-no-undefined-refs-bad)

<span id="vue-no-undefined-refs-good"></span>

[正确示例](./vue.md#vue-no-undefined-refs-good)

[分组示例](./vue.md#vue-no-undefined-refs) · [规则参考](/rules/reference/vue-no-undefined-refs.md)

### `vue/no-unsafe-url`

警告可能不安全的 URL 绑定

<span id="vue-no-unsafe-url-bad"></span>

[错误示例](./vue.md#vue-no-unsafe-url-bad)

<span id="vue-no-unsafe-url-good"></span>

[正确示例](./vue.md#vue-no-unsafe-url-good)

[分组示例](./vue.md#vue-no-unsafe-url) · [规则参考](/rules/reference/vue-no-unsafe-url.md)

### `vue/no-unsandboxed-iframe`

要求 iframe 元素具有 sandbox 属性

<span id="vue-no-unsandboxed-iframe-bad"></span>

[错误示例](./vue.md#vue-no-unsandboxed-iframe-bad)

<span id="vue-no-unsandboxed-iframe-good"></span>

[正确示例](./vue.md#vue-no-unsandboxed-iframe-good)

[分组示例](./vue.md#vue-no-unsandboxed-iframe) · [规则参考](/rules/reference/vue-no-unsandboxed-iframe.md)

### `vue/no-unused-components`

禁止注册未在模板中使用的组件

<span id="vue-no-unused-components-bad"></span>

[错误示例](./vue.md#vue-no-unused-components-bad)

<span id="vue-no-unused-components-good"></span>

[正确示例](./vue.md#vue-no-unused-components-good)

[分组示例](./vue.md#vue-no-unused-components) · [规则参考](/rules/reference/vue-no-unused-components.md)

### `vue/no-unused-properties`

禁止 defineProps 中定义的未使用属性

<span id="vue-no-unused-properties-bad"></span>

[错误示例](./vue.md#vue-no-unused-properties-bad)

<span id="vue-no-unused-properties-good"></span>

[正确示例](./vue.md#vue-no-unused-properties-good)

[分组示例](./vue.md#vue-no-unused-properties) · [规则参考](/rules/reference/vue-no-unused-properties.md)

### `vue/no-unused-refs`

报告从未在 &lt;script&gt; 中引用的模板 ref（ref="x"）

<span id="vue-no-unused-refs-bad"></span>

[错误示例](./vue.md#vue-no-unused-refs-bad)

<span id="vue-no-unused-refs-good"></span>

[正确示例](./vue.md#vue-no-unused-refs-good)

[分组示例](./vue.md#vue-no-unused-refs) · [规则参考](/rules/reference/vue-no-unused-refs.md)

### `vue/no-unused-setup-bindings`

禁止从未读取的 script setup 绑定

<span id="vue-no-unused-setup-bindings-bad"></span>

[错误示例](./vue.md#vue-no-unused-setup-bindings-bad)

<span id="vue-no-unused-setup-bindings-good"></span>

[正确示例](./vue.md#vue-no-unused-setup-bindings-good)

[分组示例](./vue.md#vue-no-unused-setup-bindings) · [规则参考](/rules/reference/vue-no-unused-setup-bindings.md)

### `vue/no-unused-vars`

禁止 v-for 和 v-slot 指令中未使用的变量定义

<span id="vue-no-unused-vars-bad"></span>

[错误示例](./vue.md#vue-no-unused-vars-bad)

<span id="vue-no-unused-vars-good"></span>

[正确示例](./vue.md#vue-no-unused-vars-good)

[分组示例](./vue.md#vue-no-unused-vars) · [规则参考](/rules/reference/vue-no-unused-vars.md)

### `vue/no-use-v-else-with-v-for`

禁止在同一元素上使用 `v-else-if` 或 `v-else` 与 `v-for`

<span id="vue-no-use-v-else-with-v-for-bad"></span>

[错误示例](./vue.md#vue-no-use-v-else-with-v-for-bad)

<span id="vue-no-use-v-else-with-v-for-good"></span>

[正确示例](./vue.md#vue-no-use-v-else-with-v-for-good)

[分组示例](./vue.md#vue-no-use-v-else-with-v-for) · [规则参考](/rules/reference/vue-no-use-v-else-with-v-for.md)

### `vue/no-use-v-if-with-v-for`

禁止在同一元素上使用 `v-if` 与 `v-for`

<span id="vue-no-use-v-if-with-v-for-bad"></span>

[错误示例](./vue.md#vue-no-use-v-if-with-v-for-bad)

<span id="vue-no-use-v-if-with-v-for-good"></span>

[正确示例](./vue.md#vue-no-use-v-if-with-v-for-good)

[分组示例](./vue.md#vue-no-use-v-if-with-v-for) · [规则参考](/rules/reference/vue-no-use-v-if-with-v-for.md)

### `vue/no-useless-mustaches`

禁止表达式为常量字符串字面量的双花括号插值

<span id="vue-no-useless-mustaches-bad"></span>

[错误示例](./vue.md#vue-no-useless-mustaches-bad)

<span id="vue-no-useless-mustaches-good"></span>

[正确示例](./vue.md#vue-no-useless-mustaches-good)

[分组示例](./vue.md#vue-no-useless-mustaches) · [规则参考](/rules/reference/vue-no-useless-mustaches.md)

### `vue/no-useless-template-attributes`

禁止在 `<template>` 元素上使用不起作用的属性

<span id="vue-no-useless-template-attributes-bad"></span>

[错误示例](./vue.md#vue-no-useless-template-attributes-bad)

<span id="vue-no-useless-template-attributes-good"></span>

[正确示例](./vue.md#vue-no-useless-template-attributes-good)

[分组示例](./vue.md#vue-no-useless-template-attributes) · [规则参考](/rules/reference/vue-no-useless-template-attributes.md)

### `vue/no-useless-v-bind`

禁止值为普通字符串字面量的 v-bind

<span id="vue-no-useless-v-bind-bad"></span>

[错误示例](./vue.md#vue-no-useless-v-bind-bad)

<span id="vue-no-useless-v-bind-good"></span>

[正确示例](./vue.md#vue-no-useless-v-bind-good)

[分组示例](./vue.md#vue-no-useless-v-bind) · [规则参考](/rules/reference/vue-no-useless-v-bind.md)

### `vue/no-v-for-template-key-on-child`

禁止在 `<template v-for>` 的子节点上使用 `key`

<span id="vue-no-v-for-template-key-on-child-bad"></span>

[错误示例](./vue.md#vue-no-v-for-template-key-on-child-bad)

<span id="vue-no-v-for-template-key-on-child-good"></span>

[正确示例](./vue.md#vue-no-v-for-template-key-on-child-good)

[分组示例](./vue.md#vue-no-v-for-template-key-on-child) · [规则参考](/rules/reference/vue-no-v-for-template-key-on-child.md)

### `vue/no-v-html`

警告使用 v-html，以避免 XSS 漏洞

<span id="vue-no-v-html-bad"></span>

[错误示例](./vue.md#vue-no-v-html-bad)

<span id="vue-no-v-html-good"></span>

[正确示例](./vue.md#vue-no-v-html-good)

[分组示例](./vue.md#vue-no-v-html) · [规则参考](/rules/reference/vue-no-v-html.md)

### `vue/no-v-text`

禁止 v-text 指令，优先使用双花括号插值

<span id="vue-no-v-text-bad"></span>

[错误示例](./vue.md#vue-no-v-text-bad)

<span id="vue-no-v-text-good"></span>

[正确示例](./vue.md#vue-no-v-text-good)

[分组示例](./vue.md#vue-no-v-text) · [规则参考](/rules/reference/vue-no-v-text.md)

### `vue/no-v-text-v-html-on-component`

禁止在组件元素上使用 v-text / v-html

<span id="vue-no-v-text-v-html-on-component-bad"></span>

[错误示例](./vue.md#vue-no-v-text-v-html-on-component-bad)

<span id="vue-no-v-text-v-html-on-component-good"></span>

[正确示例](./vue.md#vue-no-v-text-v-html-on-component-good)

[分组示例](./vue.md#vue-no-v-text-v-html-on-component) · [规则参考](/rules/reference/vue-no-v-text-v-html-on-component.md)

### `vue/permitted-contents`

要求符合 HTML 内容模型规则

<span id="vue-permitted-contents-bad"></span>

[错误示例](./vue.md#vue-permitted-contents-bad)

<span id="vue-permitted-contents-good"></span>

[正确示例](./vue.md#vue-permitted-contents-good)

[分组示例](./vue.md#vue-permitted-contents) · [规则参考](/rules/reference/vue-permitted-contents.md)

### `vue/prefer-props-shorthand`

建议使用 props 简写语法（Vue 3.4+）

<span id="vue-prefer-props-shorthand-bad"></span>

[错误示例](./vue.md#vue-prefer-props-shorthand-bad)

<span id="vue-prefer-props-shorthand-good"></span>

[正确示例](./vue.md#vue-prefer-props-shorthand-good)

[分组示例](./vue.md#vue-prefer-props-shorthand) · [规则参考](/rules/reference/vue-prefer-props-shorthand.md)

### `vue/prefer-true-attribute-shorthand`

优先使用绑定到 `true` 的布尔属性简写

<span id="vue-prefer-true-attribute-shorthand-bad"></span>

[错误示例](./vue.md#vue-prefer-true-attribute-shorthand-bad)

<span id="vue-prefer-true-attribute-shorthand-good"></span>

[正确示例](./vue.md#vue-prefer-true-attribute-shorthand-good)

[分组示例](./vue.md#vue-prefer-true-attribute-shorthand) · [规则参考](/rules/reference/vue-prefer-true-attribute-shorthand.md)

### `vue/prop-name-casing`

规范所声明 prop 名称的大小写风格

<span id="vue-prop-name-casing-bad"></span>

[错误示例](./vue.md#vue-prop-name-casing-bad)

<span id="vue-prop-name-casing-good"></span>

[正确示例](./vue.md#vue-prop-name-casing-good)

[分组示例](./vue.md#vue-prop-name-casing) · [规则参考](/rules/reference/vue-prop-name-casing.md)

### `vue/require-component-is`

要求 `<component>` 元素具有 `v-bind:is`

<span id="vue-require-component-is-bad"></span>

[错误示例](./vue.md#vue-require-component-is-bad)

<span id="vue-require-component-is-good"></span>

[正确示例](./vue.md#vue-require-component-is-good)

[分组示例](./vue.md#vue-require-component-is) · [规则参考](/rules/reference/vue-require-component-is.md)

### `vue/require-component-registration`

要求显式导入或注册组件

<span id="vue-require-component-registration-bad"></span>

[错误示例](./vue.md#vue-require-component-registration-bad)

<span id="vue-require-component-registration-good"></span>

[正确示例](./vue.md#vue-require-component-registration-good)

[分组示例](./vue.md#vue-require-component-registration) · [规则参考](/rules/reference/vue-require-component-registration.md)

### `vue/require-scoped-style`

要求 style 标签具有 scoped 属性

<span id="vue-require-scoped-style-bad"></span>

[错误示例](./vue.md#vue-require-scoped-style-bad)

<span id="vue-require-scoped-style-good"></span>

[正确示例](./vue.md#vue-require-scoped-style-good)

[分组示例](./vue.md#vue-require-scoped-style) · [规则参考](/rules/reference/vue-require-scoped-style.md)

### `vue/require-toggle-inside-transition`

要求 `<transition>` 包裹的元素具有切换条件

<span id="vue-require-toggle-inside-transition-bad"></span>

[错误示例](./vue.md#vue-require-toggle-inside-transition-bad)

<span id="vue-require-toggle-inside-transition-good"></span>

[正确示例](./vue.md#vue-require-toggle-inside-transition-good)

[分组示例](./vue.md#vue-require-toggle-inside-transition) · [规则参考](/rules/reference/vue-require-toggle-inside-transition.md)

### `vue/require-v-for-key`

要求 `v-for` 指令搭配 `v-bind:key`

<span id="vue-require-v-for-key-bad"></span>

[错误示例](./vue.md#vue-require-v-for-key-bad)

<span id="vue-require-v-for-key-good"></span>

[正确示例](./vue.md#vue-require-v-for-key-good)

[分组示例](./vue.md#vue-require-v-for-key) · [规则参考](/rules/reference/vue-require-v-for-key.md)

### `vue/scoped-event-names`

建议使用 context:event 格式的作用域事件名称

<span id="vue-scoped-event-names-bad"></span>

[错误示例](./vue.md#vue-scoped-event-names-bad)

<span id="vue-scoped-event-names-good"></span>

[正确示例](./vue.md#vue-scoped-event-names-good)

[分组示例](./vue.md#vue-scoped-event-names) · [规则参考](/rules/reference/vue-scoped-event-names.md)

### `vue/sfc-element-order`

要求 SFC 顶层元素保持一致的顺序

<span id="vue-sfc-element-order-bad"></span>

[错误示例](./vue.md#vue-sfc-element-order-bad)

<span id="vue-sfc-element-order-good"></span>

[正确示例](./vue.md#vue-sfc-element-order-good)

[分组示例](./vue.md#vue-sfc-element-order) · [规则参考](/rules/reference/vue-sfc-element-order.md)

### `vue/single-style-block`

建议只使用一个 style 块

<span id="vue-single-style-block-bad"></span>

[错误示例](./vue.md#vue-single-style-block-bad)

<span id="vue-single-style-block-good"></span>

[正确示例](./vue.md#vue-single-style-block-good)

[分组示例](./vue.md#vue-single-style-block) · [规则参考](/rules/reference/vue-single-style-block.md)

### `vue/slot-name-casing`

要求通过 v-slot 使用的具名插槽采用 kebab-case

<span id="vue-slot-name-casing-bad"></span>

[错误示例](./vue.md#vue-slot-name-casing-bad)

<span id="vue-slot-name-casing-good"></span>

[正确示例](./vue.md#vue-slot-name-casing-good)

[分组示例](./vue.md#vue-slot-name-casing) · [规则参考](/rules/reference/vue-slot-name-casing.md)

### `vue/this-in-template`

禁止在模板表达式中使用 `this.`

<span id="vue-this-in-template-bad"></span>

[错误示例](./vue.md#vue-this-in-template-bad)

<span id="vue-this-in-template-good"></span>

[正确示例](./vue.md#vue-this-in-template-good)

[分组示例](./vue.md#vue-this-in-template) · [规则参考](/rules/reference/vue-this-in-template.md)

### `vue/use-unique-element-ids`

要求使用 useId() 生成唯一元素 ID，而不是静态字面量

<span id="vue-use-unique-element-ids-bad"></span>

[错误示例](./vue.md#vue-use-unique-element-ids-bad)

<span id="vue-use-unique-element-ids-good"></span>

[正确示例](./vue.md#vue-use-unique-element-ids-good)

[分组示例](./vue.md#vue-use-unique-element-ids) · [规则参考](/rules/reference/vue-use-unique-element-ids.md)

### `vue/use-v-on-exact`

存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`

<span id="vue-use-v-on-exact-bad"></span>

[错误示例](./vue.md#vue-use-v-on-exact-bad)

<span id="vue-use-v-on-exact-good"></span>

[正确示例](./vue.md#vue-use-v-on-exact-good)

[分组示例](./vue.md#vue-use-v-on-exact) · [规则参考](/rules/reference/vue-use-v-on-exact.md)

### `vue/v-bind-style`

规范 `v-bind` 指令风格

<span id="vue-v-bind-style-bad"></span>

[错误示例](./vue.md#vue-v-bind-style-bad)

<span id="vue-v-bind-style-good"></span>

[正确示例](./vue.md#vue-v-bind-style-good)

[分组示例](./vue.md#vue-v-bind-style) · [规则参考](/rules/reference/vue-v-bind-style.md)

### `vue/v-on-event-hyphenation`

要求组件上 v-on 中的自定义事件名称使用连字符

<span id="vue-v-on-event-hyphenation-bad"></span>

[错误示例](./vue.md#vue-v-on-event-hyphenation-bad)

<span id="vue-v-on-event-hyphenation-good"></span>

[正确示例](./vue.md#vue-v-on-event-hyphenation-good)

[分组示例](./vue.md#vue-v-on-event-hyphenation) · [规则参考](/rules/reference/vue-v-on-event-hyphenation.md)

### `vue/v-on-handler-style`

要求 v-on 处理器写为方法引用或内联函数

<span id="vue-v-on-handler-style-bad"></span>

[错误示例](./vue.md#vue-v-on-handler-style-bad)

<span id="vue-v-on-handler-style-good"></span>

[正确示例](./vue.md#vue-v-on-handler-style-good)

[分组示例](./vue.md#vue-v-on-handler-style) · [规则参考](/rules/reference/vue-v-on-handler-style.md)

### `vue/v-on-style`

规范 `v-on` 指令风格

<span id="vue-v-on-style-bad"></span>

[错误示例](./vue.md#vue-v-on-style-bad)

<span id="vue-v-on-style-good"></span>

[正确示例](./vue.md#vue-v-on-style-good)

[分组示例](./vue.md#vue-v-on-style) · [规则参考](/rules/reference/vue-v-on-style.md)

### `vue/v-slot-style`

规范 `v-slot` 指令风格

<span id="vue-v-slot-style-bad"></span>

[错误示例](./vue.md#vue-v-slot-style-bad)

<span id="vue-v-slot-style-good"></span>

[正确示例](./vue.md#vue-v-slot-style-good)

[分组示例](./vue.md#vue-v-slot-style) · [规则参考](/rules/reference/vue-v-slot-style.md)

### `vue/valid-attribute-name`

要求属性名称有效

<span id="vue-valid-attribute-name-bad"></span>

[错误示例](./vue.md#vue-valid-attribute-name-bad)

<span id="vue-valid-attribute-name-good"></span>

[正确示例](./vue.md#vue-valid-attribute-name-good)

[分组示例](./vue.md#vue-valid-attribute-name) · [规则参考](/rules/reference/vue-valid-attribute-name.md)

### `vue/valid-template-root`

要求 `<template>` 根节点符合 Vue 3 片段语义

<span id="vue-valid-template-root-bad"></span>

[错误示例](./vue.md#vue-valid-template-root-bad)

<span id="vue-valid-template-root-good"></span>

[正确示例](./vue.md#vue-valid-template-root-good)

[分组示例](./vue.md#vue-valid-template-root) · [规则参考](/rules/reference/vue-valid-template-root.md)

### `vue/valid-v-bind`

要求 `v-bind` 指令有效

<span id="vue-valid-v-bind-bad"></span>

[错误示例](./vue.md#vue-valid-v-bind-bad)

<span id="vue-valid-v-bind-good"></span>

[正确示例](./vue.md#vue-valid-v-bind-good)

[分组示例](./vue.md#vue-valid-v-bind) · [规则参考](/rules/reference/vue-valid-v-bind.md)

### `vue/valid-v-cloak`

要求 `v-cloak` 指令有效

<span id="vue-valid-v-cloak-bad"></span>

[错误示例](./vue.md#vue-valid-v-cloak-bad)

<span id="vue-valid-v-cloak-good"></span>

[正确示例](./vue.md#vue-valid-v-cloak-good)

[分组示例](./vue.md#vue-valid-v-cloak) · [规则参考](/rules/reference/vue-valid-v-cloak.md)

### `vue/valid-v-else`

要求 `v-else` 指令有效

<span id="vue-valid-v-else-bad"></span>

[错误示例](./vue.md#vue-valid-v-else-bad)

<span id="vue-valid-v-else-good"></span>

[正确示例](./vue.md#vue-valid-v-else-good)

[分组示例](./vue.md#vue-valid-v-else) · [规则参考](/rules/reference/vue-valid-v-else.md)

### `vue/valid-v-else-if`

要求 `v-else-if` 指令有效

<span id="vue-valid-v-else-if-bad"></span>

[错误示例](./vue.md#vue-valid-v-else-if-bad)

<span id="vue-valid-v-else-if-good"></span>

[正确示例](./vue.md#vue-valid-v-else-if-good)

[分组示例](./vue.md#vue-valid-v-else-if) · [规则参考](/rules/reference/vue-valid-v-else-if.md)

### `vue/valid-v-for`

要求 `v-for` 指令有效

<span id="vue-valid-v-for-bad"></span>

[错误示例](./vue.md#vue-valid-v-for-bad)

<span id="vue-valid-v-for-good"></span>

[正确示例](./vue.md#vue-valid-v-for-good)

[分组示例](./vue.md#vue-valid-v-for) · [规则参考](/rules/reference/vue-valid-v-for.md)

### `vue/valid-v-html`

要求 `v-html` 指令有效

<span id="vue-valid-v-html-bad"></span>

[错误示例](./vue.md#vue-valid-v-html-bad)

<span id="vue-valid-v-html-good"></span>

[正确示例](./vue.md#vue-valid-v-html-good)

[分组示例](./vue.md#vue-valid-v-html) · [规则参考](/rules/reference/vue-valid-v-html.md)

### `vue/valid-v-if`

要求 `v-if` 指令有效

<span id="vue-valid-v-if-bad"></span>

[错误示例](./vue.md#vue-valid-v-if-bad)

<span id="vue-valid-v-if-good"></span>

[正确示例](./vue.md#vue-valid-v-if-good)

[分组示例](./vue.md#vue-valid-v-if) · [规则参考](/rules/reference/vue-valid-v-if.md)

### `vue/valid-v-is`

要求 `v-is` 指令有效

<span id="vue-valid-v-is-bad"></span>

[错误示例](./vue.md#vue-valid-v-is-bad)

<span id="vue-valid-v-is-good"></span>

[正确示例](./vue.md#vue-valid-v-is-good)

[分组示例](./vue.md#vue-valid-v-is) · [规则参考](/rules/reference/vue-valid-v-is.md)

### `vue/valid-v-memo`

要求 `v-memo` 指令有效

<span id="vue-valid-v-memo-bad"></span>

[错误示例](./vue.md#vue-valid-v-memo-bad)

<span id="vue-valid-v-memo-good"></span>

[正确示例](./vue.md#vue-valid-v-memo-good)

[分组示例](./vue.md#vue-valid-v-memo) · [规则参考](/rules/reference/vue-valid-v-memo.md)

### `vue/valid-v-model`

要求 `v-model` 指令有效

<span id="vue-valid-v-model-bad"></span>

[错误示例](./vue.md#vue-valid-v-model-bad)

<span id="vue-valid-v-model-good"></span>

[正确示例](./vue.md#vue-valid-v-model-good)

[分组示例](./vue.md#vue-valid-v-model) · [规则参考](/rules/reference/vue-valid-v-model.md)

### `vue/valid-v-on`

要求 `v-on` 指令有效

<span id="vue-valid-v-on-bad"></span>

[错误示例](./vue.md#vue-valid-v-on-bad)

<span id="vue-valid-v-on-good"></span>

[正确示例](./vue.md#vue-valid-v-on-good)

[分组示例](./vue.md#vue-valid-v-on) · [规则参考](/rules/reference/vue-valid-v-on.md)

### `vue/valid-v-once`

要求 `v-once` 指令有效

<span id="vue-valid-v-once-bad"></span>

[错误示例](./vue.md#vue-valid-v-once-bad)

<span id="vue-valid-v-once-good"></span>

[正确示例](./vue.md#vue-valid-v-once-good)

[分组示例](./vue.md#vue-valid-v-once) · [规则参考](/rules/reference/vue-valid-v-once.md)

### `vue/valid-v-pre`

要求 `v-pre` 指令有效

<span id="vue-valid-v-pre-bad"></span>

[错误示例](./vue.md#vue-valid-v-pre-bad)

<span id="vue-valid-v-pre-good"></span>

[正确示例](./vue.md#vue-valid-v-pre-good)

[分组示例](./vue.md#vue-valid-v-pre) · [规则参考](/rules/reference/vue-valid-v-pre.md)

### `vue/valid-v-show`

要求 `v-show` 指令有效

<span id="vue-valid-v-show-bad"></span>

[错误示例](./vue.md#vue-valid-v-show-bad)

<span id="vue-valid-v-show-good"></span>

[正确示例](./vue.md#vue-valid-v-show-good)

[分组示例](./vue.md#vue-valid-v-show) · [规则参考](/rules/reference/vue-valid-v-show.md)

### `vue/valid-v-slot`

要求 `v-slot` 指令有效

<span id="vue-valid-v-slot-bad"></span>

[错误示例](./vue.md#vue-valid-v-slot-bad)

<span id="vue-valid-v-slot-good"></span>

[正确示例](./vue.md#vue-valid-v-slot-good)

[分组示例](./vue.md#vue-valid-v-slot) · [规则参考](/rules/reference/vue-valid-v-slot.md)

### `vue/valid-v-text`

要求 `v-text` 指令有效

<span id="vue-valid-v-text-bad"></span>

[错误示例](./vue.md#vue-valid-v-text-bad)

<span id="vue-valid-v-text-good"></span>

[正确示例](./vue.md#vue-valid-v-text-good)

[分组示例](./vue.md#vue-valid-v-text) · [规则参考](/rules/reference/vue-valid-v-text.md)

### `vue/warn-custom-block`

警告 SFC 文件中的自定义块

<span id="vue-warn-custom-block-bad"></span>

[错误示例](./vue.md#vue-warn-custom-block-bad)

<span id="vue-warn-custom-block-good"></span>

[正确示例](./vue.md#vue-warn-custom-block-good)

[分组示例](./vue.md#vue-warn-custom-block) · [规则参考](/rules/reference/vue-warn-custom-block.md)

### `vue/warn-custom-directive`

警告需要注册的自定义指令

<span id="vue-warn-custom-directive-bad"></span>

[错误示例](./vue.md#vue-warn-custom-directive-bad)

<span id="vue-warn-custom-directive-good"></span>

[正确示例](./vue.md#vue-warn-custom-directive-good)

[分组示例](./vue.md#vue-warn-custom-directive) · [规则参考](/rules/reference/vue-warn-custom-directive.md)
