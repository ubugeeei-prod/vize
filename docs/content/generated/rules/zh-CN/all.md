---
title: "全部规则"
---

# 全部规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

本页完整展示 66 项项目检查的用途、共享项目文件，以及错误和正确示例的全部修改。两组示例都需使用各项提供的共享文件，并遵循依赖及版本说明。项目检查需要完整的已分析组件图。60 个公开跨文件诊断代码具有不同支持边界：19 个属于 CLI 分析遍（18 组已验证的源代码对照，以及一项附响应式流图的说明性 Vue 项目）；16 个有实验性 Rust 分析器实现，但该 CLI 分析遍不会单独发出这些代码；25 个仅是目前没有诊断实现的公开约定。说明性源代码和保留图的验证条件分别注明；启用规则 ID 不会激活尚不可用的实现。

公共 CLI 可通过 `vize lint --cross-file` 执行同一检查。显示的 `vize:croquis/cf/*` 代码在 `lint.vize.rules` 中写为 `croquis/cf/*`（省略 `vize:`）。信息和提示诊断会转换为 CLI 警告。相关位置说明来源与消费者之间的关系。

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
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null） |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [错误示例](#script-no-import-compiler-macros-bad) · [正确示例](#script-no-import-compiler-macros-good) | 禁止导入自动可用的 Vue 编译器宏 |
| [`script/no-internal-imports`](#script-no-internal-imports) | [错误示例](#script-no-internal-imports-bad) · [正确示例](#script-no-internal-imports-good) | 禁止从 Vue 内部模块导入 |
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

### `a11y/alt-text`

要求媒体元素提供替代文本

[错误示例](#a11y-alt-text-bad) · [正确示例](#a11y-alt-text-good)

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
        "a11y/alt-text": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-alt-text-bad"></span>

**错误示例**

图片提交控件只提供图片 URL，没有描述操作的 `alt` 文本。

```vue annotate="remove:2"
<template>
  <input type="image" src="/submit.png" />
</template>
```

<span id="a11y-alt-text-good"></span>

**正确示例**

`alt="Submit search"` 为图片控件提供描述提交搜索操作的无障碍名称。

```vue annotate="add:2"
<template>
  <input type="image" src="/submit.png" alt="Submit search" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/alt_text.rs#L33) · [全部规则](all.md)

### `a11y/anchor-has-content`

要求锚点元素具有可访问的内容

[错误示例](#a11y-anchor-has-content-bad) · [正确示例](#a11y-anchor-has-content-good)

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
        "a11y/anchor-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-has-content-bad"></span>

**错误示例**

`/settings` 链接没有文本或其他命名内容，因此其目标没有可访问的描述。

```vue annotate="remove:2"
<template>
  <a href="/settings"></a>
</template>
```

<span id="a11y-anchor-has-content-good"></span>

**正确示例**

可见的 `Settings` 文本为同一目标链接提供内容。

```vue annotate="add:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_has_content.rs#L16) · [全部规则](all.md)

### `a11y/anchor-is-valid`

要求锚点元素具有有效的 href

[错误示例](#a11y-anchor-is-valid-bad) · [正确示例](#a11y-anchor-is-valid-good)

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
        "a11y/anchor-is-valid": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-anchor-is-valid-bad"></span>

**错误示例**

第一个锚点使用 `#` 执行操作；第二个使用 JavaScript URL。两者都没有提供普通导航目标。

```vue annotate="remove:2,3"
<template>
  <a href="#" @click="openPanel">Open panel</a>
  <a href="JaVaScRiPt:void(0)">Run action</a>
</template>
```

<span id="a11y-anchor-is-valid-good"></span>

**正确示例**

原生按钮执行 `openPanel`，剩余锚点则具有真实的 `/docs/javascript-urls` 目标。

```vue annotate="add:2,3"
<template>
  <button type="button" @click="openPanel">Open panel</button>
  <a href="/docs/javascript-urls">JavaScript URL guide</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/anchor_is_valid.rs#L30) · [全部规则](all.md)

### `a11y/aria-props`

禁止无效的 ARIA 属性

[错误示例](#a11y-aria-props-bad) · [正确示例](#a11y-aria-props-good)

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
        "a11y/aria-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-props-bad"></span>

**错误示例**

`aria-lable` 拼写错误，不是受支持的 ARIA 属性。

```vue annotate="remove:2"
<template>
  <button aria-lable="Save changes">Save</button>
</template>
```

<span id="a11y-aria-props-good"></span>

**正确示例**

受支持的 `aria-label` 属性提供按钮名称。

```vue annotate="add:2"
<template>
  <button aria-label="Save changes">Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_props.rs#L18) · [全部规则](all.md)

### `a11y/aria-role`

具有 ARIA 角色的元素必须使用有效的非抽象 ARIA 角色

[错误示例](#a11y-aria-role-bad) · [正确示例](#a11y-aria-role-good)

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
        "a11y/aria-role": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-role-bad"></span>

**错误示例**

`datepicker` 不是此 section 上可识别的 ARIA 角色。

```vue annotate="remove:2"
<template>
  <section role="datepicker">...</section>
</template>
```

<span id="a11y-aria-role-good"></span>

**正确示例**

section 使用可识别的 `dialog` 角色，以及描述日期选择的标签。

```vue annotate="add:2"
<template>
  <section role="dialog" aria-label="Choose a date">...</section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_role.rs#L21) · [全部规则](all.md)

### `a11y/aria-unsupported-elements`

禁止在不支持 ARIA 的元素上使用 ARIA 属性

[错误示例](#a11y-aria-unsupported-elements-bad) · [正确示例](#a11y-aria-unsupported-elements-good)

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
        "a11y/aria-unsupported-elements": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-aria-unsupported-elements-bad"></span>

**错误示例**

元数据元素带有 `aria-hidden`，但 `meta` 不支持 ARIA 属性。

```vue annotate="remove:2"
<template>
  <meta charset="utf-8" aria-hidden="true" />
</template>
```

<span id="a11y-aria-unsupported-elements-good"></span>

**正确示例**

移除 ARIA 属性，保留字符集声明。

```vue annotate="add:2"
<template>
  <meta charset="utf-8" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/aria_unsupported_elements.rs#L18) · [全部规则](all.md)

### `a11y/click-events-have-key-events`

要求点击事件配有键盘事件处理器

[错误示例](#a11y-click-events-have-key-events-bad) · [正确示例](#a11y-click-events-have-key-events-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

检查没有交互角色的非交互式元素。原生按钮和具有交互式 ARIA 角色的元素不在此规则诊断范围内。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "a11y/click-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-click-events-have-key-events-bad"></span>

**错误示例**

非交互式 `div` 有点击处理器，却没有键盘事件处理。

```vue annotate="remove:2"
<template>
<div @click="activate">Activate</div>
</template>
```

<span id="a11y-click-events-have-key-events-good"></span>

**正确示例**

原生 `button` 为同一 `activate` 处理器提供键盘激活方式。

```vue annotate="add:2"
<template>
<button @click="activate">Activate</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/click_events_have_key_events.rs#L17) · [全部规则](all.md)

### `a11y/form-control-has-label`

要求表单控件具有相关联的标签

[错误示例](#a11y-form-control-has-label-bad) · [正确示例](#a11y-form-control-has-label-good)

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
        "a11y/form-control-has-label": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-form-control-has-label-bad"></span>

**错误示例**

搜索输入框没有说明用户应输入什么的标签。

```vue annotate="remove:2"
<template>
  <input type="search" />
</template>
```

<span id="a11y-form-control-has-label-good"></span>

**正确示例**

将输入框包在 label 中，使可见的 `Search` 文本与控件关联。

```vue annotate="add:2,3,4,5"
<template>
  <label>
    Search
    <input type="search" />
  </label>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/form_control_has_label.rs#L19) · [全部规则](all.md)

### `a11y/heading-has-content`

要求标题元素具有可访问的内容

[错误示例](#a11y-heading-has-content-bad) · [正确示例](#a11y-heading-has-content-good)

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
        "a11y/heading-has-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-has-content-bad"></span>

**错误示例**

`h2` 提供了标题层级，却没有标题内容。

```vue annotate="remove:2"
<template>
  <h2></h2>
</template>
```

<span id="a11y-heading-has-content-good"></span>

**正确示例**

`Billing settings` 为原有二级标题提供内容。

```vue annotate="add:2"
<template>
  <h2>Billing settings</h2>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/heading_has_content.rs#L17) · [全部规则](all.md)

### `a11y/heading-levels`

禁止跳过标题层级

[错误示例](#a11y-heading-levels-bad) · [正确示例](#a11y-heading-levels-good)

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
        "a11y/heading-levels": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-heading-levels-bad"></span>

**错误示例**

标题序列从 `h1` 直接跳到 `h3`，跳过二级。

```vue annotate="remove:3"
<template>
  <h1>Account</h1>
  <h3>Billing</h3>
</template>
```

<span id="a11y-heading-levels-good"></span>

**正确示例**

将账单标题改为 `h2`，保持连续的标题层级。

```vue annotate="add:3"
<template>
  <h1>Account</h1>
  <h2>Billing</h2>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/heading_levels.rs#L34) · [全部规则](all.md)

### `a11y/iframe-has-title`

要求 iframe 元素具有 title 属性

[错误示例](#a11y-iframe-has-title-bad) · [正确示例](#a11y-iframe-has-title-good)

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
        "a11y/iframe-has-title": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-iframe-has-title-bad"></span>

**错误示例**

结账框架有来源 URL，却没有描述嵌入内容的 `title`。

```vue annotate="remove:2"
<template>
  <iframe src="/checkout"></iframe>
</template>
```

<span id="a11y-iframe-has-title-good"></span>

**正确示例**

`title="Checkout preview"` 为该框架的内容命名。

```vue annotate="add:2"
<template>
  <iframe src="/checkout" title="Checkout preview"></iframe>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/iframe_has_title.rs#L15) · [全部规则](all.md)

### `a11y/img-alt`

要求图片提供 alt 属性，以支持无障碍访问

[错误示例](#a11y-img-alt-bad) · [正确示例](#a11y-img-alt-good)

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
        "a11y/img-alt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-img-alt-bad"></span>

**错误示例**

头像图片缺少 `alt` 属性。

```vue annotate="remove:2"
<template>
  <img src="/avatar.png" />
</template>
```

<span id="a11y-img-alt-good"></span>

**正确示例**

`alt="User avatar"` 为头像提供文本替代。

```vue annotate="add:2"
<template>
  <img src="/avatar.png" alt="User avatar" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/img_alt.rs#L16) · [全部规则](all.md)

### `a11y/interactive-supports-focus`

要求具有交互角色的元素可获得焦点

[错误示例](#a11y-interactive-supports-focus-bad) · [正确示例](#a11y-interactive-supports-focus-good)

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
        "a11y/interactive-supports-focus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-interactive-supports-focus-bad"></span>

**错误示例**

为 `span` 设置按钮角色和点击处理器，并不会让它能够通过键盘获得焦点。

```vue annotate="remove:2"
<template>
  <span role="button" @click="open">Open</span>
</template>
```

<span id="a11y-interactive-supports-focus-good"></span>

**正确示例**

原生按钮可以获得焦点，并保留同一 `open` 操作。

```vue annotate="add:2"
<template>
  <button type="button" @click="open">Open</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/interactive_supports_focus.rs#L31) · [全部规则](all.md)

### `a11y/label-has-for`

要求标签具有相关联的表单控件

[错误示例](#a11y-label-has-for-bad) · [正确示例](#a11y-label-has-for-good)

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
        "a11y/label-has-for": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-label-has-for-bad"></span>

**错误示例**

独立的 label 既没有通过 `for` 关联输入框，也没有包住输入框。

```vue annotate="remove:2"
<template>
  <label>Email</label>
  <input id="email" />
</template>
```

<span id="a11y-label-has-for-good"></span>

**正确示例**

`for="email"` 与输入框 ID 一致，显式关联两个元素。

```vue annotate="add:2"
<template>
  <label for="email">Email</label>
  <input id="email" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/label_has_for.rs#L27) · [全部规则](all.md)

### `a11y/landmark-roles`

验证地标角色的位置及唯一性

[错误示例](#a11y-landmark-roles-bad) · [正确示例](#a11y-landmark-roles-good)

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
        "a11y/landmark-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-landmark-roles-bad"></span>

**错误示例**

同一模板中的两个 `main` 元素声明了重复的主地标。

```vue annotate="remove:3"
<template>
  <main>Dashboard</main>
  <main>Settings</main>
</template>
```

<span id="a11y-landmark-roles-good"></span>

**正确示例**

仪表盘保留为主地标；设置区域改为具名的导航地标。

```vue annotate="add:3"
<template>
  <main>Dashboard</main>
  <nav aria-label="Settings">...</nav>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/landmark_roles.rs#L44) · [全部规则](all.md)

### `a11y/media-has-caption`

要求媒体元素具有字幕

[错误示例](#a11y-media-has-caption-bad) · [正确示例](#a11y-media-has-caption-good)

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
        "a11y/media-has-caption": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-media-has-caption-bad"></span>

**错误示例**

视频有播放控件，却没有字幕轨道。

```vue annotate="remove:2"
<template>
  <video src="/demo.mp4" controls />
</template>
```

<span id="a11y-media-has-caption-good"></span>

**正确示例**

带有 `kind="captions"` 的 `track` 为同一视频提供英文字幕。

```vue annotate="add:2,3,4"
<template>
  <video src="/demo.mp4" controls>
    <track kind="captions" src="/demo.en.vtt" srclang="en" label="English" />
  </video>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/media_has_caption.rs#L30) · [全部规则](all.md)

### `a11y/mouse-events-have-key-events`

要求鼠标事件配有 focus/blur 事件

[错误示例](#a11y-mouse-events-have-key-events-bad) · [正确示例](#a11y-mouse-events-have-key-events-good)

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
        "a11y/mouse-events-have-key-events": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-mouse-events-have-key-events-bad"></span>

**错误示例**

预览可见性只通过鼠标进入和离开处理器改变。

```vue annotate="remove:2"
<template>
  <div @mouseenter="showPreview" @mouseleave="hidePreview">Preview</div>
</template>
```

<span id="a11y-mouse-events-have-key-events-good"></span>

**正确示例**

相同的预览操作也在 focus 和 blur 时执行，并且按钮可以获得键盘焦点。

```vue annotate="add:2,3,4,5,6,7,8,9,10"
<template>
  <button
    type="button"
    @focus="showPreview"
    @blur="hidePreview"
    @mouseenter="showPreview"
    @mouseleave="hidePreview"
  >
    Preview
  </button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/mouse_events_have_key_events.rs#L30) · [全部规则](all.md)

### `a11y/no-access-key`

禁止使用 accesskey 属性

[错误示例](#a11y-no-access-key-bad) · [正确示例](#a11y-no-access-key-good)

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
        "a11y/no-access-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-access-key-bad"></span>

**错误示例**

`accesskey="s"` 快捷键可能与浏览器或辅助技术快捷键冲突。

```vue annotate="remove:2"
<template>
  <button accesskey="s">Save</button>
</template>
```

<span id="a11y-no-access-key-good"></span>

**正确示例**

移除 `accesskey`，普通的 Save 按钮仍可使用。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_access_key.rs#L19) · [全部规则](all.md)

### `a11y/no-aria-hidden-on-focusable`

禁止在可获得焦点的元素上使用 aria-hidden="true"

[错误示例](#a11y-no-aria-hidden-on-focusable-bad) · [正确示例](#a11y-no-aria-hidden-on-focusable-good)

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
        "a11y/no-aria-hidden-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-aria-hidden-on-focusable-bad"></span>

**错误示例**

可获得焦点的 Close 按钮使用 `aria-hidden="true"`，从无障碍树中隐藏。

```vue annotate="remove:2"
<template>
  <button aria-hidden="true" @click="close">Close</button>
</template>
```

<span id="a11y-no-aria-hidden-on-focusable-good"></span>

**正确示例**

按钮继续对辅助技术可见，并获得 `Close` 标签，不再被隐藏。

```vue annotate="add:2"
<template>
  <button aria-label="Close" @click="close">Close</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_aria_hidden_on_focusable.rs#L19) · [全部规则](all.md)

### `a11y/no-autofocus`

禁止使用 autofocus 属性

[错误示例](#a11y-no-autofocus-bad) · [正确示例](#a11y-no-autofocus-good)

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
        "a11y/no-autofocus": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-autofocus-bad"></span>

**错误示例**

输入框在出现时请求自动获得焦点。

```vue annotate="remove:2"
<template>
  <input autofocus name="query" />
</template>
```

<span id="a11y-no-autofocus-good"></span>

**正确示例**

移除 `autofocus`，避免该自动焦点请求，同时保留查询输入框。

```vue annotate="add:2"
<template>
  <input name="query" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_autofocus.rs#L19) · [全部规则](all.md)

### `a11y/no-distracting-elements`

禁止 &lt;marquee&gt; 和 &lt;blink&gt; 等分散注意力的元素

[错误示例](#a11y-no-distracting-elements-bad) · [正确示例](#a11y-no-distracting-elements-good)

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
        "a11y/no-distracting-elements": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-distracting-elements-bad"></span>

**错误示例**

`marquee` 元素引入了自动移动的文本。

```vue annotate="remove:2"
<template>
  <marquee>Limited offer</marquee>
</template>
```

<span id="a11y-no-distracting-elements-good"></span>

**正确示例**

段落显示同样的优惠信息，不使用分散注意力的 marquee 元素。

```vue annotate="add:2"
<template>
  <p>Limited offer</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_distracting_elements.rs#L16) · [全部规则](all.md)

### `a11y/no-i-for-icon`

禁止使用 &lt;i&gt; 元素表示图标

[错误示例](#a11y-no-i-for-icon-bad) · [正确示例](#a11y-no-i-for-icon-good)

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
        "a11y/no-i-for-icon": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-i-for-icon-bad"></span>

**错误示例**

图标通过 `i` 渲染，但其文本语义无法描述仅有图标的操作。

```vue annotate="remove:3"
<template>
  <button>
    <i class="material-icons">delete</i>
  </button>
</template>
```

<span id="a11y-no-i-for-icon-good"></span>

**正确示例**

装饰性 span 隐藏图标字形，独立的 `Delete item` 文本为按钮操作命名。

```vue annotate="add:3,4"
<template>
  <button>
    <span class="material-icons" aria-hidden="true">delete</span>
    <span class="sr-only">Delete item</span>
  </button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_i_for_icon.rs#L35) · [全部规则](all.md)

### `a11y/no-redundant-roles`

禁止多余的 ARIA 角色

[错误示例](#a11y-no-redundant-roles-bad) · [正确示例](#a11y-no-redundant-roles-good)

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
        "a11y/no-redundant-roles": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-redundant-roles-bad"></span>

**错误示例**

原生按钮已具有按钮角色，因此 `role="button"` 重复了其隐式语义。

```vue annotate="remove:2"
<template>
  <button role="button">Save</button>
</template>
```

<span id="a11y-no-redundant-roles-good"></span>

**正确示例**

移除重复角色，保留 HTML 提供的按钮语义。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_redundant_roles/report.rs#L31) · [全部规则](all.md)

### `a11y/no-refer-to-non-existent-id`

禁止引用不存在的 ID

[错误示例](#a11y-no-refer-to-non-existent-id-bad) · [正确示例](#a11y-no-refer-to-non-existent-id-good)

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
        "a11y/no-refer-to-non-existent-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-refer-to-non-existent-id-bad"></span>

**错误示例**

`aria-labelledby` 指向 `save-label`，但没有元素声明该 ID。

```vue
<template>
  <button aria-labelledby="save-label">Save</button>
</template>
```

<span id="a11y-no-refer-to-non-existent-id-good"></span>

**正确示例**

添加匹配的 span，解决引用问题并提供按钮标签。

```vue annotate="add:2"
<template>
  <span id="save-label">Save changes</span>
  <button aria-labelledby="save-label">Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_refer_to_non_existent_id.rs#L37) · [全部规则](all.md)

### `a11y/no-role-presentation-on-focusable`

禁止在可获得焦点的元素上使用 role="presentation" 或 role="none"

[错误示例](#a11y-no-role-presentation-on-focusable-bad) · [正确示例](#a11y-no-role-presentation-on-focusable-good)

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
        "a11y/no-role-presentation-on-focusable": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-role-presentation-on-focusable-bad"></span>

**错误示例**

可获得焦点的账单链接请求 role=presentation，与其交互式链接角色冲突；浏览器必须忽略该呈现角色请求。

```vue annotate="remove:2"
<template>
  <a href="/billing" role="presentation">Billing</a>
</template>
```

<span id="a11y-no-role-presentation-on-focusable-good"></span>

**正确示例**

移除冲突的呈现角色请求，使用原生链接角色和账单目标。

```vue annotate="add:2"
<template>
  <a href="/billing">Billing</a>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_role_presentation_on_focusable.rs#L19) · [全部规则](all.md)

### `a11y/no-static-element-interactions`

禁止在静态元素上设置事件处理器

[错误示例](#a11y-no-static-element-interactions-bad) · [正确示例](#a11y-no-static-element-interactions-good)

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
        "a11y/no-static-element-interactions": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-no-static-element-interactions-bad"></span>

**错误示例**

静态 section 接收 Enter 键操作，却没有交互角色。

```vue annotate="remove:2"
<template>
  <section @keydown.enter="select">Select</section>
</template>
```

<span id="a11y-no-static-element-interactions-good"></span>

**正确示例**

原生按钮以适当的交互元素承载相同操作。

```vue annotate="add:2"
<template>
  <button type="button" @keydown.enter="select">Select</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/no_static_element_interactions.rs#L31) · [全部规则](all.md)

### `a11y/placeholder-label-option`

要求 select 的占位选项具有 disabled 或 hidden

[错误示例](#a11y-placeholder-label-option-bad) · [正确示例](#a11y-placeholder-label-option-good)

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
        "a11y/placeholder-label-option": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-placeholder-label-option-bad"></span>

**错误示例**

空值提示仍可被选择，仿佛它是一个国家值。

```vue annotate="remove:3"
<template>
  <select v-model="country">
    <option value="">Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

<span id="a11y-placeholder-label-option-good"></span>

**正确示例**

添加 `disabled`，将提示与可选择的 Japan 选项区分开。

```vue annotate="add:3"
<template>
  <select v-model="country">
    <option value="" disabled>Choose a country</option>
    <option value="jp">Japan</option>
  </select>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/placeholder_label_option.rs#L36) · [全部规则](all.md)

### `a11y/role-has-required-aria-props`

要求 ARIA 角色具有必需属性

[错误示例](#a11y-role-has-required-aria-props-bad) · [正确示例](#a11y-role-has-required-aria-props-good)

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
        "a11y/role-has-required-aria-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-role-has-required-aria-props-bad"></span>

**错误示例**

checkbox 角色省略了表达复选框状态的 `aria-checked`。

```vue annotate="remove:2"
<template>
  <span role="checkbox">Receive updates</span>
</template>
```

<span id="a11y-role-has-required-aria-props-good"></span>

**正确示例**

`aria-checked="false"` 提供 checkbox 角色所需的状态。

```vue annotate="add:2"
<template>
  <span role="checkbox" aria-checked="false">Receive updates</span>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/role_has_required_aria_props.rs#L30) · [全部规则](all.md)

### `a11y/tabindex-no-positive`

禁止正数 tabindex 值

[错误示例](#a11y-tabindex-no-positive-bad) · [正确示例](#a11y-tabindex-no-positive-good)

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
        "a11y/tabindex-no-positive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-tabindex-no-positive-bad"></span>

**错误示例**

正数 tabindex 3 建立了位于普通控件之前的自定义焦点顺序。

```vue annotate="remove:2"
<template>
  <button tabindex="3">Save</button>
</template>
```

<span id="a11y-tabindex-no-positive-good"></span>

**正确示例**

按钮使用原生焦点顺序，不设置正数 tabindex。

```vue annotate="add:2"
<template>
  <button>Save</button>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/a11y/tabindex_no_positive.rs#L16) · [全部规则](all.md)

### `a11y/use-list`

建议为类似项目符号的文本使用列表元素

[错误示例](#a11y-use-list-bad) · [正确示例](#a11y-use-list-good)

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
        "a11y/use-list": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="a11y-use-list-bad"></span>

**错误示例**

任务项使用单独的段落和手写短横线标记，没有使用列表元素。

```vue annotate="remove:2,3"
<template>
  <p>- First task</p>
  <p>- Second task</p>
</template>
```

<span id="a11y-use-list-good"></span>

**正确示例**

无序列表及列表项以列表语义表达相同任务。

```vue annotate="add:2,3,4,5"
<template>
  <ul>
    <li>First task</li>
    <li>Second task</li>
  </ul>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/a11y/use_list.rs#L36) · [全部规则](all.md)

### `css/no-display-none`

建议使用 v-show 代替 display: none

[错误示例](#css-no-display-none-bad) · [正确示例](#css-no-display-none-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-display-none": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-display-none-bad"></span>

**错误示例**

`.message` 声明通过 CSS 隐藏当前段落，没有使用模板可见性条件。

```vue annotate="remove:2,4,5,6,7,8,9"
<template>
  <p class="message">Saved</p>
</template>

<style scoped>
.message {
  display: none;
}
</style>
```

<span id="css-no-display-none-good"></span>

**正确示例**

`v-show="isSaved"` 在当前段落上显式表达可见性条件，并移除 `display: none`。

```vue annotate="add:2"
<template>
  <p v-show="isSaved" class="message">Saved</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_display_none.rs#L27) · [全部规则](all.md)

### `css/no-hardcoded-values`

建议使用 CSS 变量代替硬编码值

[错误示例](#css-no-hardcoded-values-bad) · [正确示例](#css-no-hardcoded-values-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-hardcoded-values": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-hardcoded-values-bad"></span>

**错误示例**

按钮将间距数字和十六进制颜色直接写入声明。

```vue annotate="remove:3,4"
<style scoped>
.button {
  padding: 12px 16px;
  color: #174ea6;
}
</style>
```

<span id="css-no-hardcoded-values-good"></span>

**正确示例**

声明引用具名的间距和颜色自定义属性，使这些值可作为设计令牌维护。

```vue annotate="add:3,4"
<style scoped>
.button {
  padding: var(--space-3) var(--space-4);
  color: var(--color-action-text);
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_hardcoded_values.rs#L30) · [全部规则](all.md)

### `css/no-id-selectors`

不建议在 CSS 中使用 ID 选择器

[错误示例](#css-no-id-selectors-bad) · [正确示例](#css-no-id-selectors-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-id-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-id-selectors-bad"></span>

**错误示例**

`#submit` 将样式规则绑定到 ID 选择器。

```vue annotate="remove:2"
<style scoped>
#submit {
  font-weight: 600;
}
</style>
```

<span id="css-no-id-selectors-good"></span>

**正确示例**

`.submit` 类提供可复用的样式入口，不使用 ID 选择器。

```vue annotate="add:2"
<style scoped>
.submit {
  font-weight: 600;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_id_selectors.rs#L19) · [全部规则](all.md)

### `css/no-important`

不建议在 CSS 中使用 !important

[错误示例](#css-no-important-bad) · [正确示例](#css-no-important-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-important": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-important-bad"></span>

**错误示例**

颜色声明通过 `!important` 覆盖普通层叠优先级。

```vue annotate="remove:3"
<style scoped>
.button {
  color: red !important;
}
</style>
```

<span id="css-no-important-good"></span>

**正确示例**

颜色来自自定义属性，没有使用 important 声明。

```vue annotate="add:3"
<style scoped>
.button {
  color: var(--button-color);
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_important.rs#L14) · [全部规则](all.md)

### `css/no-utility-classes`

警告在组件样式中实现工具类

[错误示例](#css-no-utility-classes-bad) · [正确示例](#css-no-utility-classes-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-utility-classes": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-utility-classes-bad"></span>

**错误示例**

编写的选择器采用 `.flex`、`.mt-4` 和 `.text-center` 等工具类形式的名称。

```vue annotate="remove:2,3,4"
<style scoped>
.flex { display: flex; }
.mt-4 { margin-top: 1rem; }
.text-center { text-align: center; }
</style>
```

<span id="css-no-utility-classes-good"></span>

**正确示例**

组件专用的 `.my-component` 选择器以一个语义名称组织组件样式。

```vue annotate="add:2"
<style scoped>
.my-component { display: flex; margin-top: 1rem; }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_utility_classes.rs#L37) · [全部规则](all.md)

### `css/no-v-bind-performance`

警告 CSS v-bind() 的性能开销

[错误示例](#css-no-v-bind-performance-bad) · [正确示例](#css-no-v-bind-performance-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/no-v-bind-performance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-no-v-bind-performance-bad"></span>

**错误示例**

样式表通过 SFC CSS 的 `v-bind()` 机制读取变化中的 `offset`。

```vue annotate="remove:1,2,3,4,5"
<style scoped>
.card {
  transform: translateX(v-bind(offset));
}
</style>
```

<span id="css-no-v-bind-performance-good"></span>

**正确示例**

元素通过自身的样式绑定直接接收变化中的 transform。

```vue annotate="add:1,2,3"
<template>
  <article :style="{ transform: `translateX(${offset}px)` }" class="card" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/no_v_bind_performance.rs#L20) · [全部规则](all.md)

### `css/prefer-logical-properties`

建议使用 CSS 逻辑属性，以更好地支持国际化

[错误示例](#css-prefer-logical-properties-bad) · [正确示例](#css-prefer-logical-properties-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-logical-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-logical-properties-bad"></span>

**错误示例**

`margin-left` 无论书写方向如何，都将外边距固定在物理侧。

```vue annotate="remove:3"
<style scoped>
.panel {
  margin-left: 1rem;
}
</style>
```

<span id="css-prefer-logical-properties-good"></span>

**正确示例**

`margin-inline-start` 改为跟随行内方向的起始侧。

```vue annotate="add:3"
<style scoped>
.panel {
  margin-inline-start: 1rem;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_logical_properties.rs#L15) · [全部规则](all.md)

### `css/prefer-nested-selectors`

建议为后代选择器使用 CSS 嵌套

[错误示例](#css-prefer-nested-selectors-bad) · [正确示例](#css-prefer-nested-selectors-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-nested-selectors": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-nested-selectors-bad"></span>

**错误示例**

`.card .title` 后代选择器在平铺规则中重复了父选择器。

```vue annotate="remove:2"
<style scoped>
.card .title { color: red; }
</style>
```

<span id="css-prefer-nested-selectors-good"></span>

**正确示例**

`.title` 规则嵌套在 `.card` 内，将父子样式关系放在一起。

```vue annotate="add:2"
<style scoped>
.card { .title { color: red; } }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_nested_selectors.rs#L14) · [全部规则](all.md)

### `css/prefer-slotted`

建议使用 ::v-slotted() 为插槽内容设置样式

[错误示例](#css-prefer-slotted-bad) · [正确示例](#css-prefer-slotted-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/prefer-slotted": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-prefer-slotted-bad"></span>

**错误示例**

scoped 样式表选择的是 `slot` 出口，而不是通过插槽传入的元素。

```vue annotate="remove:2"
<style scoped>
slot { color: red; }
</style>
```

<span id="css-prefer-slotted-good"></span>

**正确示例**

`:slotted(.label)` 通过作用域插槽选择器选中传入的 label 元素。

```vue annotate="add:2"
<style scoped>
:slotted(.label) { color: red; }
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/prefer_slotted.rs#L34) · [全部规则](all.md)

### `css/require-font-display`

要求 @font-face 规则具有 font-display

[错误示例](#css-require-font-display-bad) · [正确示例](#css-require-font-display-good)

默认严重程度: `warning`  
预设: `opinionated`, `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: SFC style 块内的 CSS  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "css/require-font-display": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="css-require-font-display-bad"></span>

**错误示例**

font-face 声明定义了字体来源，却省略了 font-display 策略。

```vue
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
}
</style>
```

<span id="css-require-font-display-good"></span>

**正确示例**

`font-display: swap` 显式选择先显示后备字体、再切换到目标字体的策略。

```vue annotate="add:5"
<style>
@font-face {
  font-family: "Inter";
  src: url("/inter.woff2") format("woff2");
  font-display: swap;
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/css/require_font_display.rs#L13) · [全部规则](all.md)

### `ecosystem/nuxt-prefer-nuxt-link`

应用内部链接优先使用 NuxtLink

[错误示例](#ecosystem-nuxt-prefer-nuxt-link-bad) · [正确示例](#ecosystem-nuxt-prefer-nuxt-link-good)

默认严重程度: `warning`  
预设: `nuxt`  
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
        "ecosystem/nuxt-prefer-nuxt-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-nuxt-prefer-nuxt-link-bad"></span>

**错误示例**

Nuxt 应用中的内部设置目标使用了普通锚点。

```vue annotate="remove:2"
<template>
  <a href="/settings">Settings</a>
</template>
```

<span id="ecosystem-nuxt-prefer-nuxt-link-good"></span>

**正确示例**

NuxtLink 通过 Nuxt 路由处理相同的内部目标。

```vue annotate="add:2"
<template>
  <NuxtLink to="/settings">Settings</NuxtLink>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/nuxt_prefer_nuxt_link.rs#L14) · [全部规则](all.md)

### `ecosystem/pinia-prefer-store-to-refs`

解构 Pinia store 时优先使用 storeToRefs()

[错误示例](#ecosystem-pinia-prefer-store-to-refs-bad) · [正确示例](#ecosystem-pinia-prefer-store-to-refs-good)

默认严重程度: `warning`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/pinia-prefer-store-to-refs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-pinia-prefer-store-to-refs-bad"></span>

**错误示例**

直接从 store 解构 `name`，使该值脱离了响应式 store 访问。

```vue annotate="remove:2"
<script setup lang="ts">
const { name } = useUserStore();
</script>
```

<span id="ecosystem-pinia-prefer-store-to-refs-good"></span>

**正确示例**

store 保持完整，storeToRefs 为 name 创建响应式引用。

```vue annotate="add:2,3"
<script setup lang="ts">
const store = useUserStore();
const { name } = storeToRefs(store);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/pinia_prefer_store_to_refs.rs#L20) · [全部规则](all.md)

### `ecosystem/router-link-require-to`

要求 RouterLink 和 NuxtLink 组件具有 `to` 目标

[错误示例](#ecosystem-router-link-require-to-bad) · [正确示例](#ecosystem-router-link-require-to-good)

默认严重程度: `error`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

SFC 的唯一根链接可能从父组件属性继承目标。此示例使用嵌套链接，必须显式提供目标。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/router-link-require-to": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-router-link-require-to-bad"></span>

**错误示例**

嵌套 RouterLink 没有 `to` 目标，不能依赖根节点的属性透传。

```vue annotate="remove:2"
<template>
<nav><RouterLink>Settings</RouterLink></nav>
</template>
```

<span id="ecosystem-router-link-require-to-good"></span>

**正确示例**

`to="/settings"` 显式提供嵌套链接目标。

```vue annotate="add:2"
<template>
<nav><RouterLink to="/settings">Settings</RouterLink></nav>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/router_link_require_to.rs#L14) · [全部规则](all.md)

### `ecosystem/void-link-require-href`

要求 Void Vue Link 组件具有 `href`

[错误示例](#ecosystem-void-link-require-href-bad) · [正确示例](#ecosystem-void-link-require-href-good)

默认严重程度: `error`  
预设: `ecosystem`  
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
        "ecosystem/void-link-require-href": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-require-href-bad"></span>

**错误示例**

从 @void/vue 导入的 Link 省略了 href 目标。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link>Settings</Link>
</template>
```

<span id="ecosystem-void-link-require-href-good"></span>

**正确示例**

同一导入的 Link 通过 href 接收设置目标。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/settings">Settings</Link>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_require_href.rs#L13) · [全部规则](all.md)

### `ecosystem/void-link-valid-method`

验证 Void Vue Link 的静态 method prop

[错误示例](#ecosystem-void-link-valid-method-bad) · [正确示例](#ecosystem-void-link-valid-method-good)

默认严重程度: `warning`  
预设: `ecosystem`  
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
        "ecosystem/void-link-valid-method": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-void-link-valid-method-bad"></span>

**错误示例**

DELETE 操作请求了预取，但预取适用于导航请求。

```vue annotate="remove:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE" prefetch>Delete</Link>
</template>
```

<span id="ecosystem-void-link-valid-method-good"></span>

**正确示例**

移除 prefetch，保留 DELETE 操作，不再预取该非 GET 请求。

```vue annotate="add:6"
<script setup>
import { Link } from "@void/vue";
</script>

<template>
  <Link href="/posts/1" method="DELETE">Delete</Link>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/void_link_valid_method.rs#L14) · [全部规则](all.md)

### `ecosystem/vue-i18n-no-missing-key`

报告本地 SFC 消息中不存在的静态 vue-i18n 键

[错误示例](#ecosystem-vue-i18n-no-missing-key-bad) · [正确示例](#ecosystem-vue-i18n-no-missing-key-good)

默认严重程度: `warning`  
预设: `ecosystem`  
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
        "ecosystem/vue-i18n-no-missing-key": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-i18n-no-missing-key-bad"></span>

**错误示例**

模板请求 auth.missing，但本地英文消息只声明了 auth.login。

```vue annotate="remove:1"
<template>{{ $t("auth.missing") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

<span id="ecosystem-vue-i18n-no-missing-key-good"></span>

**正确示例**

模板请求本地消息中已存在的 auth.login 键。

```vue annotate="add:1"
<template>{{ $t("auth.login") }}</template>

<i18n lang="json">
{ "en": { "auth": { "login": "Log in" } } }
</i18n>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/i18n_no_missing_key.rs#L17) · [全部规则](all.md)

### `ecosystem/vue-router-extra-param`

路由未声明 tab；Vue Router 会丢弃它。

默认严重程度: error  
适用范围: 可达的项目声明及导入组件  
选项: crossFile；规则严重程度（off/warn/error）  
自动修复: 无

完整且已安装的路由器必须从应用的 createApp(...).use(router) 可达。未知或动态路由表不能证明未知名称诊断。缺失参数属于警告，因为导航可能从当前路由继承值。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-extra-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-extra-param-bad"></span>

**错误示例**

`user-post` 路径声明 `userId` 和 `postId`，但导航还将未声明的 `tab` 作为路径参数传入。

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2", tab: "a" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-extra-param-good"></span>

**正确示例**

从 params 移除 `tab`，只保留路由路径中的键。应用需要选择标签页时，应另行使用 query。

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[跨文件规则索引](cross-file.md)

### `ecosystem/vue-router-missing-param`

缺少必需的 postId；依赖当前路由并不稳健。

默认严重程度: warning  
适用范围: 可达的项目声明及导入组件  
选项: crossFile；规则严重程度（off/warn/error）  
自动修复: 无

完整且已安装的路由器必须从应用的 createApp(...).use(router) 可达。未知或动态路由表不能证明未知名称诊断。缺失参数属于警告，因为导航可能从当前路由继承值。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-missing-param": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-missing-param-bad"></span>

**错误示例**

导航省略了 `user-post` 路径必需的 `postId`。这是警告，因为 Vue Router 可能从当前路由继承该值。

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-missing-param-good"></span>

**正确示例**

显式传入 `userId` 和 `postId`，使导航不依赖当前路由的参数状态。

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[跨文件规则索引](cross-file.md)

### `ecosystem/vue-router-param-type`

postId 不是可重复参数，因此数组无效。

默认严重程度: error  
适用范围: 可达的项目声明及导入组件  
选项: crossFile；规则严重程度（off/warn/error）  
自动修复: 无

完整且已安装的路由器必须从应用的 createApp(...).use(router) 可达。未知或动态路由表不能证明未知名称诊断。缺失参数属于警告，因为导航可能从当前路由继承值。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-param-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-param-type-bad"></span>

**错误示例**

`postId` 是标量路径参数，但导航传入了数组 `["2"]`。

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: ["2"] } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-param-type-good"></span>

**正确示例**

为不可重复的 `postId` 路径段传入标量 `"2"`。

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[跨文件规则索引](cross-file.md)

### `ecosystem/vue-router-prefer-named-link`

RouterLink 优先使用具名路由对象，而不是静态路径字符串

[错误示例](#ecosystem-vue-router-prefer-named-link-bad) · [正确示例](#ecosystem-vue-router-prefer-named-link-good)

默认严重程度: `warning`  
预设: `ecosystem`  
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
        "ecosystem/vue-router-prefer-named-link": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-link-bad"></span>

**错误示例**

RouterLink 的目标是路径字面量，而不是具名路由。

```vue annotate="remove:2"
<template>
  <RouterLink to="/settings">Settings</RouterLink>
</template>
```

<span id="ecosystem-vue-router-prefer-named-link-good"></span>

**正确示例**

绑定的路由对象通过 settings 路由名称标识目标。

```vue annotate="add:2"
<template>
  <RouterLink :to="{ name: 'settings' }">Settings</RouterLink>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ecosystem/vue_router_prefer_named_link.rs#L15) · [全部规则](all.md)

### `ecosystem/vue-router-prefer-named-push`

Vue Router 编程式导航优先使用具名路由对象

[错误示例](#ecosystem-vue-router-prefer-named-push-bad) · [正确示例](#ecosystem-vue-router-prefer-named-push-good)

默认严重程度: `warning`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-router-prefer-named-push": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-router-prefer-named-push-bad"></span>

**错误示例**

router.push 接收与当前 URL 写法绑定的路径字符串。

```vue annotate="remove:2"
<script setup lang="ts">
router.push("/settings");
</script>
```

<span id="ecosystem-vue-router-prefer-named-push-good"></span>

**正确示例**

router.push 接收具有稳定 settings 名称的路由对象。

```vue annotate="add:2"
<script setup lang="ts">
router.push({ name: "settings" });
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_router_prefer_named_push.rs#L19) · [全部规则](all.md)

### `ecosystem/vue-router-unknown-route`

名称不在完整且已安装的路由器中。

默认严重程度: error  
适用范围: 可达的项目声明及导入组件  
选项: crossFile；规则严重程度（off/warn/error）  
自动修复: 无

完整且已安装的路由器必须从应用的 createApp(...).use(router) 可达。未知或动态路由表不能证明未知名称诊断。缺失参数属于警告，因为导航可能从当前路由继承值。

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "ecosystem/vue-router-unknown-route": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`index.html`

```html
<div id="app"></div>
<script type="module" src="/src/main.ts"></script>
```

`src/main.ts`

```ts
import { createApp } from "vue";
import App from "./App.vue";
import { router } from "./router";
createApp(App).use(router).mount("#app");
```

`src/App.vue`

```vue
<script setup lang="ts">
import { RouterView } from "vue-router";
</script>
<template><RouterView /></template>
```

`src/router.ts`

```ts
import { createRouter, createWebHistory } from "vue-router";
import UserPost from "./UserPost.vue";
export const router = createRouter({
  history: createWebHistory(),
  routes: [{ path: "/users/:userId/posts/:postId", name: "user-post", component: UserPost }],
});
```

<span id="ecosystem-vue-router-unknown-route-bad"></span>

**错误示例**

可达且已安装的路由器声明了 `user-post`，但导航使用拼写错误的 `user-posts`。

`src/UserPost.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-posts", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

<span id="ecosystem-vue-router-unknown-route-good"></span>

**正确示例**

使用已注册的 `user-post` 名称，并保留两个已声明的路径参数。

`src/UserPost.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { useRouter } from "vue-router";
const router = useRouter();
router.push({ name: "user-post", params: { userId: "1", postId: "2" } });
</script>
<template><p>Post</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[跨文件规则索引](cross-file.md)

### `ecosystem/vue-test-utils-no-html-snapshot`

避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照

[错误示例](#ecosystem-vue-test-utils-no-html-snapshot-bad) · [正确示例](#ecosystem-vue-test-utils-no-html-snapshot-good)

默认严重程度: `warning`  
预设: `ecosystem`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "ecosystem/vue-test-utils-no-html-snapshot": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-bad"></span>

**错误示例**

断言对整个 wrapper HTML 生成快照，而不是检查预期行为。

```vue annotate="remove:2"
<script setup lang="ts">
expect(wrapper.html()).toMatchSnapshot();
</script>
```

<span id="ecosystem-vue-test-utils-no-html-snapshot-good"></span>

**正确示例**

断言检查渲染文本是否包含 Saved。

```vue annotate="add:2"
<script setup lang="ts">
expect(wrapper.text()).toContain("Saved");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/vue_test_utils_no_html_snapshot.rs#L15) · [全部规则](all.md)

### `html/cross-component-nesting`

检查导入组件组合后的实际 HTML 嵌套。

默认严重程度: warning  
适用范围: 可达的项目声明及导入组件  
选项: crossFile；规则严重程度（off/warn/error）  
自动修复: 无

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "html/cross-component-nesting": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="html-cross-component-nesting-bad"></span>

**错误示例**

父组件的 `<p>` 包含根为 `<div>` 的已解析子组件，组合后形成无效的段落与块级元素嵌套。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><p><Child /></p></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

<span id="html-cross-component-nesting-good"></span>

**正确示例**

改用可容纳子组件块级元素的 `<section>` 容器；子组件保持不变。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><section><Child /></section></template>
```

`Child.vue`

```vue
<template><div>Block content</div></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[跨文件规则索引](cross-file.md)

### `html/deprecated-attr`

禁止已弃用的 HTML 属性

[错误示例](#html-deprecated-attr-bad) · [正确示例](#html-deprecated-attr-good)

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
        "html/deprecated-attr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-attr-bad"></span>

**错误示例**

段落使用了已弃用的呈现属性 `align`。

```vue annotate="remove:1,2,3"
<template>
<p align="center">Notice</p>
</template>
```

<span id="html-deprecated-attr-good"></span>

**正确示例**

类名和 `text-align: center` 声明通过 CSS 表达对齐方式。

```vue annotate="add:1,2"
<template><p class="notice">Notice</p></template>
<style scoped>.notice { text-align: center; }</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_attr.rs#L32) · [全部规则](all.md)

### `html/deprecated-element`

禁止已弃用的 HTML 元素

[错误示例](#html-deprecated-element-bad) · [正确示例](#html-deprecated-element-good)

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
        "html/deprecated-element": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-deprecated-element-bad"></span>

**错误示例**

`center` 使用了已弃用的 HTML 呈现元素。

```vue annotate="remove:2"
<template>
  <center>Profile</center>
</template>
```

<span id="html-deprecated-element-good"></span>

**正确示例**

section 和样式类替代弃用元素，同时保留内容。

```vue annotate="add:2"
<template>
  <section class="profile">Profile</section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/deprecated_element.rs#L33) · [全部规则](all.md)

### `html/id-duplication`

禁止重复的元素 ID

[错误示例](#html-id-duplication-bad) · [正确示例](#html-id-duplication-good)

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
        "html/id-duplication": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-id-duplication-bad"></span>

**错误示例**

输入框和帮助段落都声明 `id="email"`，使标签目标不明确。

```vue annotate="remove:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" />
  <p id="email">Required</p>
</template>
```

<span id="html-id-duplication-good"></span>

**正确示例**

输入框保留 `email`；帮助段落使用 `email-help`，aria-describedby 引用该独立 ID。

```vue annotate="add:3,4"
<template>
  <label for="email">Email</label>
  <input id="email" aria-describedby="email-help" />
  <p id="email-help">Required</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/id_duplication.rs#L36) · [全部规则](all.md)

### `html/no-consecutive-br`

禁止连续的 &lt;br&gt; 元素

[错误示例](#html-no-consecutive-br-bad) · [正确示例](#html-no-consecutive-br-good)

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
        "html/no-consecutive-br": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-consecutive-br-bad"></span>

**错误示例**

两个连续换行元素在同一段落内为内容块制造间距。

```vue annotate="remove:2"
<template>
  <p>First line<br /><br />Second block</p>
</template>
```

<span id="html-no-consecutive-br-good"></span>

**正确示例**

使用独立段落表达两个内容块，无需重复换行元素。

```vue annotate="add:2,3"
<template>
  <p>First line</p>
  <p>Second block</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_consecutive_br.rs#L30) · [全部规则](all.md)

### `html/no-dupe-style-properties`

禁止内联 style 属性中重复的属性声明

[错误示例](#html-no-dupe-style-properties-bad) · [正确示例](#html-no-dupe-style-properties-good)

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
        "html/no-dupe-style-properties": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-dupe-style-properties-bad"></span>

**错误示例**

每个静态 style 都重复了一个属性；`margin` 和 `MARGIN` 也算作同一属性。

```vue annotate="remove:2,3"
<template>
<div style="color: red; color: blue">text</div>
<div style="margin: 0; MARGIN: 1px">text</div>
</template>
```

<span id="html-no-dupe-style-properties-good"></span>

**正确示例**

静态 style 使用不同的 color 和 background 属性。动态样式绑定不在此静态属性检查范围内。

```vue annotate="add:2,3"
<template>
<div style="color: red; background: blue">text</div>
<div :style="{ color: a, color: b }">text</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_dupe_style_properties.rs#L37) · [全部规则](all.md)

### `html/no-duplicate-class`

禁止静态 class 属性中重复的类名

[错误示例](#html-no-duplicate-class-bad) · [正确示例](#html-no-duplicate-class-good)

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
        "html/no-duplicate-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-class-bad"></span>

**错误示例**

静态类名列表重复了 `btn`。

```vue annotate="remove:2"
<template>
<div class="btn btn primary">click</div>
</template>
```

<span id="html-no-duplicate-class-good"></span>

**正确示例**

类名列表保留一个 `btn` 及不同的 `primary`。

```vue annotate="add:2"
<template>
<div class="btn primary">click</div>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/html/no_duplicate_class.rs#L32) · [全部规则](all.md)

### `html/no-duplicate-dt`

禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称

[错误示例](#html-no-duplicate-dt-bad) · [正确示例](#html-no-duplicate-dt-good)

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
        "html/no-duplicate-dt": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-duplicate-dt-bad"></span>

**错误示例**

同一定义列表为两项描述重复了 `API` 术语。

```vue annotate="remove:5"
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dt>API</dt>
    <dd>Internal service</dd>
  </dl>
</template>
```

<span id="html-no-duplicate-dt-good"></span>

**正确示例**

一个 API 术语后跟两项描述，避免重复术语。

```vue
<template>
  <dl>
    <dt>API</dt>
    <dd>Public interface</dd>
    <dd>Internal service</dd>
  </dl>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_duplicate_dt.rs#L41) · [全部规则](all.md)

### `html/no-empty-palpable-content`

禁止预期具有可见内容的空元素

[错误示例](#html-no-empty-palpable-content-bad) · [正确示例](#html-no-empty-palpable-content-good)

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
        "html/no-empty-palpable-content": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-no-empty-palpable-content-bad"></span>

**错误示例**

段落、列表项和表格单元格都没有可感知的内容。

```vue annotate="remove:2,3,4"
<template>
  <p></p>
  <li></li>
  <td></td>
</template>
```

<span id="html-no-empty-palpable-content-good"></span>

**正确示例**

文本填充段落，插值提供列表项内容，aria-label 则显式命名原本为空的单元格。

```vue annotate="add:2,3,4"
<template>
  <p>Overview</p>
  <li>{{ item.label }}</li>
  <td aria-label="No value"></td>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/no_empty_palpable_content.rs#L32) · [全部规则](all.md)

### `html/require-datetime`

要求 &lt;time&gt; 元素具有 datetime 属性

[错误示例](#html-require-datetime-bad) · [正确示例](#html-require-datetime-good)

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
        "html/require-datetime": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="html-require-datetime-bad"></span>

**错误示例**

time 元素包含人类可读的日期，却没有机器可读的 datetime 值。

```vue annotate="remove:2"
<template>
  <time>May 13, 2026</time>
</template>
```

<span id="html-require-datetime-good"></span>

**正确示例**

`datetime="2026-05-13"` 提供对应的机器可读日期。

```vue annotate="add:2"
<template>
  <time datetime="2026-05-13">May 13, 2026</time>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/html/require_datetime.rs#L34) · [全部规则](all.md)

### `musea/no-empty-variant`

禁止空的 &lt;variant&gt; 块

[错误示例](#musea-no-empty-variant-bad) · [正确示例](#musea-no-empty-variant-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/no-empty-variant": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-no-empty-variant-bad"></span>

**错误示例**

名为 primary 的 variant 为空，没有提供预览内容。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-no-empty-variant-good"></span>

**正确示例**

variant 渲染带有 Save 内容的 primary Button。

```vue annotate="add:2,3,4"
<art title="Button" component="./Button.vue">
  <variant name="primary">
    <Button tone="primary">Save</Button>
  </variant>
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/no_empty_variant.rs#L8) · [全部规则](all.md)

### `musea/prefer-design-tokens`

优先使用设计令牌 CSS 变量，而不是硬编码的原始值

[错误示例](#musea-prefer-design-tokens-bad) · [正确示例](#musea-prefer-design-tokens-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 参见[类型化选项和默认值](/rules/options.md)。

需要 .art.vue 文件及下方所示的令牌清单。不会从任意颜色推断设计令牌。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/prefer-design-tokens": "warn"
      },
      "ruleOptions": {
        "musea/prefer-design-tokens": {
          "tokens": [
            {
              "path": "color.primary",
              "value": "#3b82f6",
              "tier": "semantic"
            }
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

<span id="musea-prefer-design-tokens-bad"></span>

**错误示例**

art 示例使用蓝色字面量，而不是配置的 primary 设计令牌。

`Button.art.vue`

```vue annotate="remove:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: #3b82f6;
}
</style>
```

<span id="musea-prefer-design-tokens-good"></span>

**正确示例**

样式引用此示例配置的令牌 --color-primary。

`Button.art.vue`

```vue annotate="add:6"
<art title="Button" component="Button">
<variant name="Primary"><Button /></variant>
</art>
<style scoped>
.button {
  color: var(--color-primary);
}
</style>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/prefer_design_tokens.rs#L32) · [全部规则](all.md)

### `musea/require-component`

要求 &lt;art&gt; 块具有 component 属性

[错误示例](#musea-require-component-bad) · [正确示例](#musea-require-component-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-component": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-component-bad"></span>

**错误示例**

art 块提供了标题，却未标识要预览的组件。

```vue annotate="remove:1"
<art title="Button">
  <variant name="primary" />
</art>
```

<span id="musea-require-component-good"></span>

**正确示例**

defineArt 将 ./Button.vue 指定为 art 块的组件。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_component.rs#L11) · [全部规则](all.md)

### `musea/require-title`

要求 &lt;art&gt; 块具有 title 属性

[错误示例](#musea-require-title-bad) · [正确示例](#musea-require-title-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/require-title": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-require-title-bad"></span>

**错误示例**

art 块标识了 Button.vue，却没有提供标题。

```vue annotate="remove:1"
<art component="./Button.vue">
  <variant name="primary" />
</art>
```

<span id="musea-require-title-good"></span>

**正确示例**

defineArt 选项为 art 块提供 Button 标题。

```vue annotate="add:1,2,3,4,5"
<script setup>
defineArt("./Button.vue", { title: "Button" });
</script>

<art>
  <variant name="primary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/require_title.rs#L31) · [全部规则](all.md)

### `musea/unique-variant-names`

要求 variant 名称唯一

[错误示例](#musea-unique-variant-names-bad) · [正确示例](#musea-unique-variant-names-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/unique-variant-names": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-unique-variant-names-bad"></span>

**错误示例**

同一 art 块中的两个 variant 都使用 primary 名称。

```vue annotate="remove:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="primary" />
</art>
```

<span id="musea-unique-variant-names-good"></span>

**正确示例**

两个 variant 分别使用不同的 primary 和 secondary 名称。

```vue annotate="add:3"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
  <variant name="secondary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/unique_variant_names.rs#L10) · [全部规则](all.md)

### `musea/valid-variant`

要求 &lt;variant&gt; 块具有 name 属性

[错误示例](#musea-valid-variant-bad) · [正确示例](#musea-valid-variant-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Musea .art.vue 的 art、variant 和 style 块  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "musea/valid-variant": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="musea-valid-variant-bad"></span>

**错误示例**

variant 省略了标识预览所需的名称。

```vue annotate="remove:2"
<art title="Button" component="./Button.vue">
  <variant />
</art>
```

<span id="musea-valid-variant-good"></span>

**正确示例**

primary 名称标识了该 variant。

```vue annotate="add:2"
<art title="Button" component="./Button.vue">
  <variant name="primary" />
</art>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/musea/valid_variant.rs#L8) · [全部规则](all.md)

### `nuxt/no-nuxt-config-test-key`

禁止在 Nuxt 配置中设置 `test` 键

[错误示例](#nuxt-no-nuxt-config-test-key-bad) · [正确示例](#nuxt-no-nuxt-config-test-key-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: Nuxt 配置文件（nuxt.config.ts）  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-nuxt-config-test-key": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-nuxt-config-test-key-bad"></span>

**错误示例**

导出的 Nuxt 配置将标识符键 `test` 设为布尔值 `true`，这是此规则拒绝的旧配置形式。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ test: true });
```

<span id="nuxt-no-nuxt-config-test-key-good"></span>

**正确示例**

空配置移除了该布尔 `test` 属性。此示例并不禁止测试配置对象。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({});
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_nuxt_config_test_key.rs#L16) · [全部规则](all.md)

### `nuxt/no-page-meta-runtime-values`

禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行

[错误示例](#nuxt-no-page-meta-runtime-values-bad) · [正确示例](#nuxt-no-page-meta-runtime-values-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/no-page-meta-runtime-values": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-no-page-meta-runtime-values-bad"></span>

**错误示例**

构造 `definePageMeta` 对象时立即执行 `useRoute()`，但宏会将元数据提升到 setup 运行时上下文之外。

```vue annotate="remove:2"
<script setup lang="ts">
definePageMeta({ title: useRoute() });
</script>
```

<span id="nuxt-no-page-meta-runtime-values-good"></span>

**正确示例**

`validate` 接收回调，因此 `useRoute().params.id` 访问延迟到回调运行时。规则区分延迟执行的函数体和立即求值的元数据值。

```vue annotate="add:2"
<script setup lang="ts">
definePageMeta({ validate: () => Boolean(useRoute().params.id) });
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_page_meta_runtime_values.rs#L25) · [全部规则](all.md)

### `nuxt/nuxt-config-keys-order`

优先使用推荐的 Nuxt 配置属性顺序

[错误示例](#nuxt-nuxt-config-keys-order-bad) · [正确示例](#nuxt-nuxt-config-keys-order-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 适用于已支持的诊断  
适用范围: Nuxt 配置文件（nuxt.config.ts）  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/nuxt-config-keys-order": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-nuxt-config-keys-order-bad"></span>

**错误示例**

配置将 `ssr` 放在 `modules` 之前，与规则推荐的 Nuxt 配置键顺序相反。

`nuxt.config.ts`

```ts annotate="remove:1"
export default defineNuxtConfig({ ssr: true, modules: [] });
```

<span id="nuxt-nuxt-config-keys-order-good"></span>

**正确示例**

将 `modules` 放在 `ssr` 之前，在保留两个值的同时满足规定顺序；修复改变布局，不改变选项含义。

`nuxt.config.ts`

```ts annotate="add:1"
export default defineNuxtConfig({ modules: [], ssr: true });
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/nuxt_config_keys_order.rs#L24) · [全部规则](all.md)

### `nuxt/prefer-import-meta`

优先使用 `import.meta.*`，而不是 `process.*`

[错误示例](#nuxt-prefer-import-meta-bad) · [正确示例](#nuxt-prefer-import-meta-good)

默认严重程度: `error`  
预设: `nuxt`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "nuxt/prefer-import-meta": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="nuxt-prefer-import-meta-bad"></span>

**错误示例**

`process.client` 使用旧 Nuxt 环境标志，规则要求将其迁移到 `import.meta`。

```vue annotate="remove:2"
<script setup lang="ts">
if (process.client) console.log("browser");
</script>
```

<span id="nuxt-prefer-import-meta-good"></span>

**正确示例**

`import.meta.client` 通过替代环境标志显式保留仅浏览器执行的分支。

```vue annotate="add:2"
<script setup lang="ts">
if (import.meta.client) console.log("browser");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_meta.rs#L18) · [全部规则](all.md)

### `petite-vue/no-unsupported-directive`

禁止 petite-vue 不支持的指令

[错误示例](#petite-vue-no-unsupported-directive-bad) · [正确示例](#petite-vue-no-unsupported-directive-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/no-unsupported-directive": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-no-unsupported-directive-bad"></span>

**错误示例**

`v-memo`、`v-slot:header` 和自定义 `v-my-directive` 不在 petite-vue 支持的指令列表中。petite-vue 脚本将此 HTML 标识为相应方言。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-memo="[a, b]"></div>
<template v-slot:header></template>
<div v-my-directive></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-no-unsupported-directive-good"></span>

**正确示例**

替换后使用受支持的 `v-scope`、`v-effect`、`v-if`、`v-bind` 和 `v-on` 语法，不再依赖不支持的指令。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-scope="{ count: 0 }" v-effect="console.log(count)"></div>
<div v-if="ok" v-bind:title="title" @click="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/no_unsupported_directive.rs#L43) · [全部规则](all.md)

### `petite-vue/valid-v-effect`

要求 v-effect 具有非空表达式

[错误示例](#petite-vue-valid-v-effect-bad) · [正确示例](#petite-vue-valid-v-effect-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-effect": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-effect-bad"></span>

**错误示例**

每个 `v-effect` 都没有可执行表达式：值缺失、为空，或只包含空白。

```html annotate="remove:3,4,5"
<!doctype html>
<html><body>
<div v-effect></div>
<div v-effect=""></div>
<div v-effect="   "></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-effect-good"></span>

**正确示例**

两个 `v-effect` 值都包含表达式：一个更新 `el.textContent`，另一个递增 `count`。规则检查表达式非空，不检查副作用的业务逻辑。

```html annotate="add:3,4"
<!doctype html>
<html><body>
<div v-effect="el.textContent = count"></div>
<div v-effect="count++"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_effect.rs#L35) · [全部规则](all.md)

### `petite-vue/valid-v-scope`

要求 v-scope 绑定对象字面量

[错误示例](#petite-vue-valid-v-scope-bad) · [正确示例](#petite-vue-valid-v-scope-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 检测为 petite-vue 的 HTML 文档；普通 Vue SFC 不在此规则范围内。  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "petite-vue/valid-v-scope": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="petite-vue-valid-v-scope-bad"></span>

**错误示例**

四个非空 `v-scope` 值分别是标识符、调用、算术表达式和数字；都不能解析为对象字面量。

```html annotate="remove:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope="count"></div>
<div v-scope="foo()"></div>
<div v-scope="a + b"></div>
<div v-scope="123"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

<span id="petite-vue-valid-v-scope-good"></span>

**正确示例**

无值的 `v-scope` 使用根作用域。其他值是对象字面量，包括规则接受的加括号对象。

```html annotate="add:3,4,5,6"
<!doctype html>
<html><body>
<div v-scope></div>
<div v-scope="{}"></div>
<div v-scope="{ count: 0 }"></div>
<div v-scope="({ count: 0 })"></div>
<script src="https://unpkg.com/petite-vue" init></script>
</body></html>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/petite_vue/valid_v_scope.rs#L46) · [全部规则](all.md)

### `script/component-options-name-casing`

要求组件 `name` 选项使用 PascalCase

[错误示例](#script-component-options-name-casing-bad) · [正确示例](#script-component-options-name-casing-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**错误示例**

组件选项 `name: 'my-component'` 使用 kebab-case，但此规则要求名称字面量采用 PascalCase。

```vue annotate="remove:3"
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**正确示例**

`MyComponent` 以大写字母开头，且仅包含字母和数字，满足名称检查。

```vue annotate="add:3"
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [全部规则](all.md)

### `script/custom-event-name-casing`

要求发出的自定义事件名称使用 camelCase

[错误示例](#script-custom-event-name-casing-bad) · [正确示例](#script-custom-event-name-casing-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**错误示例**

发出的字符串 `my-event` 含有连字符，违反默认的 camelCase 事件命名策略。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**正确示例**

声明和调用都使用 `myEvent`，既保持事件名称与发出操作一致，也满足默认大小写策略。配置为 kebab-case 时要求不同。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [全部规则](all.md)

### `script/define-emits-declaration`

要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式

[错误示例](#script-define-emits-declaration-bad) · [正确示例](#script-define-emits-declaration-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**错误示例**

`defineEmits(["change"])` 使用运行时数组声明；此风格规则偏好类型声明。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**正确示例**

`defineEmits<{ change: [id: number] }>()` 将事件声明移到类型参数中，并显式描述 `emit("change", 1)` 使用的数字载荷。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [全部规则](all.md)

### `script/define-macros-order`

要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序

[错误示例](#script-define-macros-order-bad) · [正确示例](#script-define-macros-order-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**错误示例**

`defineProps` 位于 `defineModel` 之前，但标准宏顺序中 `defineModel` 更靠前。

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**正确示例**

声明严格按照 `defineOptions`、`defineModel`、`defineProps`、`defineEmits`、`defineSlots` 排列，再写无关的运行时语句。

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [全部规则](all.md)

### `script/define-props-declaration`

要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式

[错误示例](#script-define-props-declaration-bad) · [正确示例](#script-define-props-declaration-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**错误示例**

`defineProps({ title: String })` 提供运行时对象，与此规则偏好的类型 props 声明冲突。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**正确示例**

`defineProps<{ title: string }>()` 在类型参数中声明 `title`，保留 `props.title` 访问，无需运行时声明参数。

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [全部规则](all.md)

### `script/define-props-destructuring`

要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致

[错误示例](#script-define-props-destructuring-bad) · [正确示例](#script-define-props-destructuring-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**错误示例**

`defineProps` 被赋给单一的 `props` 绑定，而非解构，不符合默认的解构偏好。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**正确示例**

对象模式直接绑定 `foo` 和 `bar`，并为可选的 `bar` 提供默认值。这依赖 Vue 3.5+ 的响应式 props 解构；可配置的 `never` 模式偏好相反形式。

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [全部规则](all.md)

### `script/no-arrow-functions-in-watch`

禁止将箭头函数用作 Options API watch 处理器

[错误示例](#script-no-arrow-functions-in-watch-bad) · [正确示例](#script-no-arrow-functions-in-watch-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**错误示例**

Options API 侦听器 `value` 和嵌套的 `other.handler` 都是箭头函数。箭头函数捕获外层 `this`，不会接收组件实例。

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
watch: {
// `this` is not the component instance inside an arrow function.
value: () => {
this.doSomething()
},
other: {
handler: () => {}
}
}
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**正确示例**

两个处理器改为普通方法，使 Vue 能将 `this` 绑定到组件。`deep: true` 侦听选项仍与对象形式兼容。

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
watch: {
value(newValue, oldValue) {
this.doSomething()
},
other: {
handler(newValue) {},
deep: true
}
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [全部规则](all.md)

### `script/no-async-in-computed`

禁止计算属性中的异步函数

[错误示例](#script-no-async-in-computed-bad) · [正确示例](#script-no-async-in-computed-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**错误示例**

`computed` getter 使用 `async`，因此 fetch 产生 Promise，而不是同步派生的计算值。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**正确示例**

异步 fetch 移到 `watch`，将结果存入 `data.value`。清理逻辑中止旧请求，并阻止失效回调写入过期结果；不再存在异步计算 getter。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [全部规则](all.md)

### `script/no-boolean-default`

禁止为 Boolean prop 设置默认值

[错误示例](#script-no-boolean-default-bad) · [正确示例](#script-no-boolean-default-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**错误示例**

`disabled` 和 `checked` 都为唯一构造器为 `Boolean` 的 prop 声明了 `default`；规则连显式的 `false` 默认值也会拒绝。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**正确示例**

仅 Boolean 的 props 省略 `default`，使用 Vue 隐式的 false 值。`[Boolean, String]` 联合类型和 Number prop 说明此检查仅限唯一的 `Boolean` 构造器。

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [全部规则](all.md)

### `script/no-deep-destructure-in-props`

禁止在 defineProps 中进行深层嵌套解构

[错误示例](#script-no-deep-destructure-in-props-bad) · [正确示例](#script-no-deep-destructure-in-props-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**错误示例**

绑定模式进入 `user` 解构 `name`，超过默认的浅层 props 解构深度。

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**正确示例**

props 对象保持完整，由计算 getter 读取 `props.user.name`。嵌套访问保持显式，不使用深层嵌套绑定模式。

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [全部规则](all.md)

### `script/no-deprecated-data-object-declaration`

禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）

[错误示例](#script-no-deprecated-data-object-declaration-bad) · [正确示例](#script-no-deprecated-data-object-declaration-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**错误示例**

Options API 的 `data` 选项是对象字面量，这是 Vue 3 不再接受的 Vue 2 形式。

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**正确示例**

`data()` 返回新的 `{ count: 0 }` 对象，提供 Vue 3 所需的函数式 data 声明。

```vue annotate="add:3,4"
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [全部规则](all.md)

### `script/no-deprecated-destroyed-lifecycle`

禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子

[错误示例](#script-no-deprecated-destroyed-lifecycle-bad) · [正确示例](#script-no-deprecated-destroyed-lifecycle-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**错误示例**

定时器清理使用了已移除的 Vue 2 生命周期选项 `beforeDestroy`。

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**正确示例**

将钩子重命名为 `beforeUnmount`，在 Vue 3 生命周期名称下保留清理函数体。

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [全部规则](all.md)

### `script/no-deprecated-dollar-listeners-api`

禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）

[错误示例](#script-no-deprecated-dollar-listeners-api-bad) · [正确示例](#script-no-deprecated-dollar-listeners-api-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**错误示例**

成员读取和独立参数引用都使用 `$listeners`，Vue 3 在将监听器合并到属性后移除了它。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**正确示例**

读取改为 `this.$attrs` 和 setup 上下文的 `ctx.attrs`，替代已移除的监听器接口；示例中的接收对象必须存在于周围组件上下文。

```vue annotate="add:2,3"
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [全部规则](all.md)

### `script/no-deprecated-dollar-scopedslots-api`

禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）

[错误示例](#script-no-deprecated-dollar-scopedslots-api-bad) · [正确示例](#script-no-deprecated-dollar-scopedslots-api-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**错误示例**

`this.$scopedSlots`、`ctx.$scopedSlots` 和独立的 `$scopedSlots` 引用使用了 Vue 3 已移除的 Vue 2 作用域插槽 API。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**正确示例**

将 `$scopedSlots` 替换为 `$slots`，使用统一插槽接口。示例移除弃用写法，并未为接收对象建立 setup 上下文。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [全部规则](all.md)

### `script/no-deprecated-events-api`

禁止已移除的 Vue 2 事件 API（$on / $off / $once）

[错误示例](#script-no-deprecated-events-api-bad) · [正确示例](#script-no-deprecated-events-api-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**错误示例**

`$on`、`$once` 和 `$off` 调用使用了 Vue 3 已移除的实例事件总线方法。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**正确示例**

`$emit` 仍然有效，事件总线订阅则移到外部发射器的 `on` 方法。修复区分面向父组件的事件发出与外部事件总线。

```vue annotate="add:2,3,4,5,6,7,8"
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [全部规则](all.md)

### `script/no-deprecated-props-default-this`

禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）

[错误示例](#script-no-deprecated-props-default-this-bad) · [正确示例](#script-no-deprecated-props-default-this-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**错误示例**

prop 默认值和校验函数读取 `this`，但在 Vue 3 中这些函数不能依赖组件实例。

```vue annotate="remove:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// `this` is not the component instance in Vue 3.
default() {
return this.defaultSize
}
},
value: {
type: Number,
validator() {
return this.value > 0
}
}
}
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**正确示例**

默认值从参数读取 `props.baseSize`，校验函数检查 `value` 参数。两者都不再依赖不可用的实例接收对象。

```vue annotate="add:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// Vue 3 passes the raw props as the first argument instead.
default(props) {
return props.baseSize
}
},
value: {
type: Number,
validator(value) {
return value > 0
}
}
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [全部规则](all.md)

### `script/no-dupe-keys`

禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键

[错误示例](#script-no-dupe-keys-bad) · [正确示例](#script-no-dupe-keys-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**错误示例**

props 和 data 都声明 `foo`，computed 和 methods 都声明 `bar`。这些声明竞争相同的组件实例键。

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { foo: 1 } // duplicate of prop `foo`
},
computed: {
bar() { return 2 }
},
methods: {
bar() {} // duplicate of computed `bar`
}
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**正确示例**

prop、data 和 computed 声明使用不同名称（`foo`、`bar` 和 `baz`），消除两处跨选项冲突。

```vue annotate="add:5,8"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { bar: 1 }
},
computed: {
baz() { return 2 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [全部规则](all.md)

### `script/no-duplicate-attr-inheritance`

标记重复应用透传属性的组件

[错误示例](#script-no-duplicate-attr-inheritance-bad) · [正确示例](#script-no-duplicate-attr-inheritance-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**错误示例**

显式 `inheritAttrs: true` 重复了 Vue 的默认值。即使没有展示根节点 `$attrs` 展开，规则也会报告此多余字面量。

```vue annotate="remove:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**正确示例**

`inheritAttrs: false` 表达实际的退出选择，空选项对象则隐式保留默认继承。两者都不重复多余的 `true`。

```vue annotate="add:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [全部规则](all.md)

### `script/no-export-in-script-setup`

禁止 &lt;script setup&gt; 内的 export 语句

[错误示例](#script-no-export-in-script-setup-bad) · [正确示例](#script-no-export-in-script-setup-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**错误示例**

`export const count` 试图从 `<script setup>` 暴露模块导出，但这里禁止运行时导出。

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**正确示例**

移除 `export`，使 `count` 保持为 setup 绑定，而非模块导出。

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [全部规则](all.md)

### `script/no-get-current-instance`

禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）

[错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good)

默认严重程度: `error`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**错误示例**

带 Vapor 标记的 setup 导入并调用 `getCurrentInstance`，依赖了此规则针对 Vapor 组件禁止的实例 API。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**正确示例**

`inject("app-config")` 获取显式提供的配置，无需导入或调用 `getCurrentInstance`。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全部规则](all.md)

### `script/no-import-compiler-macros`

禁止导入自动可用的 Vue 编译器宏

[错误示例](#script-no-import-compiler-macros-bad) · [正确示例](#script-no-import-compiler-macros-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**错误示例**

`vue` 导入包含 `defineProps` 和 `defineEmits`，但这些编译器宏已在 `<script setup>` 中直接可用。

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**正确示例**

移除宏导入，保留两个带类型的宏调用；两种声明都无需运行时导入。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [全部规则](all.md)

### `script/no-internal-imports`

禁止从 Vue 内部模块导入

[错误示例](#script-no-internal-imports-bad) · [正确示例](#script-no-internal-imports-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**错误示例**

两个导入都指向内部 `dist` 文件，而非 Vue 公共包入口，使组件与构建文件路径耦合。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**正确示例**

从 `vue` 导入所需辅助函数，移除对内部发行文件位置的依赖。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [全部规则](all.md)

### `script/no-multiple-slot-args`

禁止向作用域插槽函数调用传入多个参数

[错误示例](#script-no-multiple-slot-args-bad) · [正确示例](#script-no-multiple-slot-args-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**错误示例**

插槽调用传入多个位置参数，或展开未知参数列表。Vue 插槽接收一个 props 对象，而非位置参数列表。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**正确示例**

`{ foo, bar }` 将数据合并为一个参数；`slotProps` 和无参数调用也符合支持的插槽调用形式。

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [全部规则](all.md)

### `script/no-next-tick`

禁止面向 Vapor 的组件使用 nextTick()

[错误示例](#script-no-next-tick-bad) · [正确示例](#script-no-next-tick-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**错误示例**

面向 Vapor 的组件导入并等待 `nextTick`，引入此迁移规则拒绝的 DOM 刷新调度依赖。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**正确示例**

通过 `useTemplateRef` 获取输入框，并在 `onMounted` 时聚焦。显式挂载边界替代示例中的 `nextTick` 依赖。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全部规则](all.md)

### `script/no-options-api`

禁止 Vapor 模式中的 Options API 模式

[错误示例](#script-no-options-api-bad) · [正确示例](#script-no-options-api-good)

默认严重程度: `error`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**错误示例**

默认导出对象声明了 Options API 的 `data()`，这是此规则禁止的组件选项形式。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**正确示例**

组件状态改为 Vapor `<script setup>` 中的 Composition API `ref`，移除 Options API 对象及其 `data` 选项。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全部规则](all.md)

### `script/no-potential-component-option-typo`

标记 Options API 组件选项名称中的疑似拼写错误

[错误示例](#script-no-potential-component-option-typo-bad) · [正确示例](#script-no-potential-component-option-typo-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**错误示例**

选项被写为 `method`，与受识别的 `methods` 选项只差一次编辑；Vue 不会将它视为预期的方法声明。

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**正确示例**

将键改为 `methods`，使 `save()` 位于受识别的组件选项下。

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [全部规则](all.md)

### `script/no-reactive-destructure`

禁止导致响应性丢失的响应式对象解构

[错误示例](#script-no-reactive-destructure-bad) · [正确示例](#script-no-reactive-destructure-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**错误示例**

`const { count, name } = state` 从 `reactive` 对象复制原始类型属性，失去与后续属性变化的联系。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**正确示例**

解构 `toRefs(state)` 为 `count` 和 `name` 创建 refs，使每个绑定继续关联原始响应式属性。

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [全部规则](all.md)

### `script/no-ref-as-operand`

要求将 ref 绑定变量用作操作数时通过 `.value` 访问

[错误示例](#script-no-ref-as-operand-bad) · [正确示例](#script-no-ref-as-operand-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**错误示例**

`count + 1` 将 ref 对象本身用作算术操作数，而不是其包裹的数字。

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**正确示例**

`count.value + 1` 先读取包裹的数字，再加一；脚本算术需要此显式 ref 访问。

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [全部规则](all.md)

### `script/no-required-prop-with-default`

禁止 prop 同时具有 required: true 和默认值

[错误示例](#script-no-required-prop-with-default-bad) · [正确示例](#script-no-required-prop-with-default-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**错误示例**

`title` 既是必需项，又具有后备值 `"Untitled"`，混合了必需输入约定和面向缺失输入的默认值。

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**正确示例**

移除 `required: true`，使 `title` 可选，并保留 `"Untitled"` 作为一致的后备值。

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [全部规则](all.md)

### `script/no-reserved-identifiers`

禁止使用 Vue 编译器保留标识符

[错误示例](#script-no-reserved-identifiers-bad) · [正确示例](#script-no-reserved-identifiers-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**错误示例**

绑定 `__props`、`__emit` 和 `__sfc__` 使用了为 Vue 编译器生成代码保留的标识符。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**正确示例**

普通名称 `props`、`emit` 和 `componentData` 避开这些生成标识符，同时保留 props 和 emits 声明。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [全部规则](all.md)

### `script/no-reserved-keys`

禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键

[错误示例](#script-no-reserved-keys-bad) · [正确示例](#script-no-reserved-keys-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**错误示例**

返回的 data 键 `$el` 与 Vue 内置组件实例属性冲突，也使用了保留的 `$` 前缀。

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**正确示例**

将应用数据重命名为 `elementLabel`，避开内置实例接口和保留前缀。

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [全部规则](all.md)

### `script/no-reserved-props`

禁止在组件 props 声明中使用保留名称

[错误示例](#script-no-reserved-props-bad) · [正确示例](#script-no-reserved-props-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**错误示例**

对象形式的 `ref` 和 `$foo`，以及数组形式的 `key`，都是保留 prop 名称。`ref` 和 `key` 是框架控制项，带 `$` 前缀的名称也会被拒绝。

```vue annotate="remove:4,5,6,8,9,10"
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**正确示例**

普通 prop 名称 `name` 和 `refValue` 在拼写及前缀上都避开了保留名称。

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [全部规则](all.md)

### `script/no-restricted-globals`

禁止直接引用必须通过类型化包装器访问的运行时环境全局变量

[错误示例](#script-no-restricted-globals-bad) · [正确示例](#script-no-restricted-globals-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**错误示例**

示例直接读取默认受限全局变量 `process`、`localStorage` 和 `sessionStorage`，绕过项目显式的配置和存储辅助函数。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**正确示例**

`useFeatureFlag`、`authStorage.read` 和 `viewStorage.write` 移除了这些直接引用。剩余的 `window.scrollY` 不是规则的默认限制；SSR 安全是独立问题。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [全部规则](all.md)

### `script/no-restricted-members`

禁止项目配置的 object.property 成员访问

[错误示例](#script-no-restricted-members-bad) · [正确示例](#script-no-restricted-members-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

此示例配置 window.localStorage。规则没有默认拒绝列表；仅启用规则不会报告成员。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
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

<span id="script-no-restricted-members-bad"></span>

**错误示例**

在 `ruleOptions` 中配置 `{ object: "window", property: "localStorage" }` 后，`window.localStorage` 访问了禁止的对象与成员组合。规则没有默认禁止成员。

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**正确示例**

`authStorage.read("token")` 将读取交给应用存储辅助函数，不再访问配置禁止的 `window.localStorage` 成员。

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [全部规则](all.md)

### `script/no-side-effects-in-computed-properties`

禁止 Options API 计算 getter 中的副作用

[错误示例](#script-no-side-effects-in-computed-properties-bad) · [正确示例](#script-no-side-effects-in-computed-properties-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**错误示例**

`doubled` 赋值给 `this.count`，`reversed` 通过 `reverse()` 修改 `this.items`。两个 getter 都修改了本应只用于派生的状态。

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
this.count = this.count * 2 // side effect: assigns to data
return this.count
},
reversed() {
return this.items.reverse() // side effect: mutates the array
}
}
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**正确示例**

`doubled` 返回乘法结果，不进行赋值。`reversed` 先复制数组再反转，getter 不会改变原组件状态。

```vue annotate="add:8,11"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
return this.count * 2
},
reversed() {
return [...this.items].reverse() // operate on a copy
}
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [全部规则](all.md)

### `script/no-top-level-ref-in-script`

禁止顶层 ref/reactive，以防止跨请求状态污染

[错误示例](#script-no-top-level-ref-in-script-bad) · [正确示例](#script-no-top-level-ref-in-script-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**错误示例**

普通 `<script>` 在模块作用域初始化 `count` 和 `user`。SSR 期间，这些状态对象可能跨组件实例和请求共享。

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**正确示例**

setup ref 按组件实例初始化；普通脚本只保留常量、生成状态的函数，以及 `setup()` 内创建的 ref。都不会在普通模块作用域创建响应式状态。

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [全部规则](all.md)

### `script/no-unstable-nested-components`

禁止在 setup 或 render 函数内定义组件

[错误示例](#script-no-unstable-nested-components-bad) · [正确示例](#script-no-unstable-nested-components-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**错误示例**

`defineComponent` 在父组件的 `setup()` 内运行，每次执行 setup 都创建新的 `Child` 组件定义。

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**正确示例**

`Child` 定义移到模块作用域，`setup()` 返回已存在的定义。

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [全部规则](all.md)

### `script/no-unused-emit-declarations`

标记已声明却从未发出的事件

[错误示例](#script-no-unused-emit-declarations-bad) · [正确示例](#script-no-unused-emit-declarations-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**错误示例**

`defineEmits` 声明了 `change` 和 `unused`，但捕获的 `emit` 函数只发出字面量事件 `change`。

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**正确示例**

移除 `unused`，使声明的事件列表与观察到的发出操作一致。示例使用捕获且未逸出的 emit 绑定，因此可以得出此局部使用结论。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [全部规则](all.md)

### `script/no-use-computed-property-like-method`

禁止像方法一样调用 Options API 计算属性

[错误示例](#script-no-use-computed-property-like-method-bad) · [正确示例](#script-no-use-computed-property-like-method-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**错误示例**

`this.total()` 调用了计算 getter 暴露的值；getter 返回不可调用的 `3`。

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**正确示例**

`this.total` 不加调用括号，直接读取计算值，使 `log` 打印派生数字。

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [全部规则](all.md)

### `script/no-with-defaults`

不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）

[错误示例](#script-no-with-defaults-bad) · [正确示例](#script-no-with-defaults-good)

默认严重程度: `warning`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**错误示例**

`withDefaults` 包装带类型的 props 声明，仅为 `count` 和 `name` 提供默认值，没有使用此处偏好的 Vue 3.5+ 解构默认值风格。

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**正确示例**

解构模式将 `count = 0` 和 `name = "Ada"` 放在对应绑定旁，并移除 `withDefaults` 包装。

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [全部规则](all.md)

### `script/prefer-computed`

响应式派生状态优先使用 computed()

[错误示例](#script-prefer-computed-bad) · [正确示例](#script-prefer-computed-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

此情形要求侦听器仅派生目标值。可编辑副本及具有其他副作用的回调被允许。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**错误示例**

侦听器仅将 `count` 的派生值复制到第二个 ref `doubled`，通过手动同步维护派生状态。

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**正确示例**

`computed(() => count.value * 2)` 直接表达派生关系，移除额外可写 ref 及其同步侦听器。

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [全部规则](all.md)

### `script/prefer-define-options`

优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;

[错误示例](#script-prefer-define-options-bad) · [正确示例](#script-prefer-define-options-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**错误示例**

普通脚本唯一有实质作用的语句导出只含 `name` 和 `inheritAttrs` 的对象；这些选项可通过 `defineOptions` 表达。

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**正确示例**

示例中的 `data()` 方法让脚本包含实际 Options API 逻辑，因此不在此规则保守的仅选项建议范围内。此正确示例展示允许的例外；直接迁移则应在 `<script setup>` 中使用 `defineOptions({ name: 'MyComponent', inheritAttrs: false })`。

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [全部规则](all.md)

### `script/prefer-import-from-vue`

优先从 'vue' 导入，而不是内部包

[错误示例](#script-prefer-import-from-vue-bad) · [正确示例](#script-prefer-import-from-vue-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**错误示例**

`ref` 和 `h` 从内部 `@vue/runtime-core` 和 `@vue/runtime-dom` 包导入，而不是公共 `vue` 包。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**正确示例**

两个辅助函数一起从 `vue` 导入，使用公共包入口，不使用任何内部包。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [全部规则](all.md)

### `script/prefer-ref-over-reactive`

建议使用 ref() 而不是 reactive() 管理状态

[错误示例](#script-prefer-ref-over-reactive-bad) · [正确示例](#script-prefer-ref-over-reactive-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**错误示例**

状态通过 `reactive` 创建，不符合此主张型规则对 refs 的偏好。示例展示风格偏好，并非响应式对象本身无效。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**正确示例**

示例通过 `ref` 创建标量和对象状态；相关字段也可拆为独立 refs，满足偏好的状态构造形式。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [全部规则](all.md)

### `script/prefer-use-attrs`

建议使用 useAttrs() 而不是 context.attrs

[错误示例](#script-prefer-use-attrs-bad) · [正确示例](#script-prefer-use-attrs-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**错误示例**

`setup` 通过解构上下文参数获取 `attrs`，规则要求改用 Composition API 辅助函数。

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**正确示例**

`useAttrs()` 在 setup 内提供 `attrs`，保留 `attrs.class` 读取，不依赖第二个 setup 参数。

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [全部规则](all.md)

### `script/prefer-use-id`

建议使用 useId() 生成唯一 ID（Vue 3.5+）

[错误示例](#script-prefer-use-id-bad) · [正确示例](#script-prefer-use-id-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**错误示例**

`id` 包含 `Math.random()`，生成的输入框与标签标识符可能在服务端和客户端渲染时不同。以 ID 命名的绑定是规则可识别的生成上下文。

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**正确示例**

Vue 3.5+ 的 `useId()` 生成标识符，`:for` 和 `:id` 继续读取同一绑定，不再分别生成随机值。

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [全部规则](all.md)

### `script/prefer-use-slots`

建议使用 useSlots() 而不是 context.slots

[错误示例](#script-prefer-use-slots-bad) · [正确示例](#script-prefer-use-slots-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**错误示例**

`setup` 从上下文参数解构 `slots`，这是此规则建议替换的访问形式。

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**正确示例**

`useSlots()` 在 setup 内获取插槽，保留渲染函数及可选默认插槽调用，不使用上下文参数。

```vue annotate="add:2,4,5,6,7"
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [全部规则](all.md)

### `script/prefer-use-template-ref`

模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+）

[错误示例](#script-prefer-use-template-ref-bad) · [正确示例](#script-prefer-use-template-ref-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**错误示例**

可为空的 `input` ref 与模板字面量 `ref="input"` 配对，表明它是元素引用，而非普通可空数据。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
<input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**正确示例**

Vue 3.5+ 的 `useTemplateRef<HTMLInputElement>('input')` 显式表达模板引用。未配对的 `error = ref(null)` 仍是普通数据，有意不在此规则范围内。

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
<input ref="input" />
<p>{{ error }}</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [全部规则](all.md)

### `script/require-default-prop`

要求每个可选的非 Boolean prop 具有默认值

[错误示例](#script-require-default-prop-bad) · [正确示例](#script-require-default-prop-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**错误示例**

`name` 和 `age` 是没有默认值的可选非 Boolean 运行时 props，省略输入时的值未指定。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

<span id="script-require-default-prop-good"></span>

**正确示例**

`name` 获得 `default: ''`。`enabled` 使用 Boolean 隐式的 false 默认值，必需的 `id` 无需后备值，展示了两种豁免。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [全部规则](all.md)

### `script/require-explicit-emits`

要求发出的事件在 defineEmits 或 emits 选项中声明

[错误示例](#script-require-explicit-emits-bad) · [正确示例](#script-require-explicit-emits-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**错误示例**

捕获的 emit 函数发出 `save`，但 `defineEmits([])` 未声明该事件。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**正确示例**

在声明中添加 `"save"`，使发出的字面量事件成为组件显式事件约定的一部分。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [全部规则](all.md)

### `script/require-explicit-slots`

要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型

[错误示例](#script-require-explicit-slots-bad) · [正确示例](#script-require-explicit-slots-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**错误示例**

带类型的 `defineProps<{ id: number }>()` 确立了 TypeScript 语法，但 setup 使用 `useSlots()` 而没有 `defineSlots` 声明，因此规则发现缺少显式插槽约定。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**正确示例**

`defineSlots` 声明 props 含 `msg: string` 的 `default` 插槽；`useSlots()` 现在配有显式的类型化插槽约定。

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [全部规则](all.md)

### `script/require-function-return-type`

要求函数具有返回类型注解

[错误示例](#script-require-function-return-type-bad) · [正确示例](#script-require-function-return-type-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**错误示例**

`add` 和 `greet` 都注解了参数，却省略返回类型注解；推断返回类型不满足此显式注解策略。

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
return a + b
}

function greet(name: string) {
return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**正确示例**

`add` 声明 `: number`，`greet` 声明 `: string`，明确返回约定而不改变函数体。

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
return a + b
}

function greet(name: string): string {
return `Hello, ${name}`
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [全部规则](all.md)

### `script/require-prop-type-constructor`

要求 prop 的 `type` 值是构造器，而不是字符串字面量

[错误示例](#script-require-prop-type-constructor-bad) · [正确示例](#script-require-prop-type-constructor-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**错误示例**

prop 声明将字符串 `"String"` 和 `"Number"` 用作运行时类型，包括构造器数组内的值。这些字符串不是构造函数。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
// The type should be the `String` constructor, not the string "String".
name: "String",
age: { type: "Number" },
id: { type: ["String", "Number"] }
}
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**正确示例**

声明使用实际的 `String` 和 `Number` 标识符，包括联合数组 `[String, Number]`。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: String,
age: { type: Number },
id: { type: [String, Number] }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [全部规则](all.md)

### `script/require-prop-types`

要求每个 prop 声明类型

[错误示例](#script-require-prop-types-bad) · [正确示例](#script-require-prop-types-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**错误示例**

数组项只声明名称 `status`；`null` 值和空描述对象也没有声明运行时 prop 类型。

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
props: ['status']            // array form: no types
}

export default {
props: {
status: null,              // no type
other: {}                  // empty descriptor: no type
}
}
</script>
```

<span id="script-require-prop-types-good"></span>

**正确示例**

`status: String` 提供简写构造器，`other` 在描述对象内提供 `type: Number`，两个 props 现在都有类型声明。

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
status: String,
other: { type: Number, default: 0 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [全部规则](all.md)

### `script/require-symbol-provide`

建议使用 Symbol 作为 provide/inject 的注入键

[错误示例](#script-require-symbol-provide-bad) · [正确示例](#script-require-symbol-provide-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**错误示例**

`provide` 和 `inject` 使用 `'user'`、`'theme'` 等字符串字面量键，可能与同名的其他提供者冲突。

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**正确示例**

共享的 `UserKey` 通过 `Symbol` 创建，并注解为 `InjectionKey<User>`；两个调用都传入该键，而非字符串字面量。

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [全部规则](all.md)

### `script/require-typed-object-prop`

要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型

[错误示例](#script-require-typed-object-prop-bad) · [正确示例](#script-require-typed-object-prop-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**错误示例**

单独的 `Object` 和 `Array` 构造器只描述宽泛的运行时类别，因此 `user` 和 `items` 元素结构都没有显式静态类型。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**正确示例**

`PropType<User>` 和 `PropType<User[]>` 添加对象和元素类型，同时保留原运行时构造器。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [全部规则](all.md)

### `script/require-typed-ref`

要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数

[错误示例](#script-require-typed-ref-bad) · [正确示例](#script-require-typed-ref-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**错误示例**

导入的 `ref` 调用既无类型参数，也无有效初始值：无参数、`null` 和 `undefined` 都无法推断未来值的预期类型。

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**正确示例**

显式类型参数描述字符串和可空 User refs。`ref(0)` 已有具体数字初始值，可以依靠推断。

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [全部规则](all.md)

### `script/require-valid-default-prop`

要求 prop 默认值符合声明类型

[错误示例](#script-require-valid-default-prop-bad) · [正确示例](#script-require-valid-default-prop-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**错误示例**

Number 和 Boolean props 使用了不匹配的标量默认值，Array 和 Object props 使用共享字面量而不是工厂。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**正确示例**

标量默认值改为 `0` 和 `false`；数组和对象默认值改为返回新值的函数。`[String, Number]` 示例接受字符串默认值，因为它匹配一个声明类型。

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [全部规则](all.md)

### `script/return-in-computed-property`

要求每个计算 getter 返回值

[错误示例](#script-return-in-computed-property-bad) · [正确示例](#script-return-in-computed-property-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**错误示例**

块函数体计算 getter 求值 `1 + 2` 却从不返回，使计算值为 undefined。

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**正确示例**

`return 1 + 2` 将表达式转为 getter 的返回值。规则查找 getter 自身返回值的 return，而非仅有表达式语句。

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [全部规则](all.md)

### `script/return-in-emits-validator`

要求每个 Options API emits 校验函数返回值

[错误示例](#script-return-in-emits-validator-bad) · [正确示例](#script-return-in-emits-validator-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

为当前支持的 SFC 筛选使用块函数体箭头函数。底层校验器也处理方法简写，但当前 SFC 前置筛选无法可靠分派该形式。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**错误示例**

`submit` 校验函数记录载荷，却不返回校验结果，因此块函数体产生 undefined。

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**正确示例**

`return payload != null` 为提交载荷提供布尔校验结果，避免结束时没有返回值。

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [全部规则](all.md)

### `script/valid-define-emits`

要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）

[错误示例](#script-valid-define-emits-bad) · [正确示例](#script-valid-define-emits-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**错误示例**

同一 `defineEmits` 调用同时提供类型参数和运行时数组 `["save"]`，混合两种互斥声明。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**正确示例**

移除运行时参数，只保留一个针对 `save` 的类型事件声明。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [全部规则](all.md)

### `script/valid-define-options`

要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots）

[错误示例](#script-valid-define-options-bad) · [正确示例](#script-valid-define-options-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**错误示例**

第一个调用将专用的 `props` 声明放入 `defineOptions`；后续调用又重复宏，并包含非对象参数。这些展示了禁止的形式和重复调用限制。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**正确示例**

一个 `defineOptions` 调用接收只包含受支持普通选项 `name` 和 `inheritAttrs` 的对象。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [全部规则](all.md)

### `script/valid-define-props`

要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）

[错误示例](#script-valid-define-props-bad) · [正确示例](#script-valid-define-props-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**错误示例**

同一 `defineProps` 调用同时提供类型参数 `{ title: string }` 和运行时参数 `{ title: String }`，编译器不允许二者组合。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**正确示例**

移除运行时对象，只保留一个针对 `title` 的类型声明，不再组合两种声明形式。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [全部规则](all.md)

### `script/valid-next-tick`

要求 nextTick() 调用结果被等待、链式处理，或提供回调

[错误示例](#script-valid-next-tick-bad) · [正确示例](#script-valid-next-tick-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**错误示例**

导入的 `nextTick()` 是无回调的独立表达式，其返回 Promise 被忽略，没有操作等待 DOM 刷新。

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**正确示例**

`await nextTick()` 使用该 Promise，显式等待下一次 DOM 更新，再继续执行后续 setup 代码。

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [全部规则](all.md)

### `ssr/no-browser-globals-in-ssr`

禁止 SSR 上下文中的浏览器专用全局变量

[错误示例](#ssr-no-browser-globals-in-ssr-bad) · [正确示例](#ssr-no-browser-globals-in-ssr-good)

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
        "ssr/no-browser-globals-in-ssr": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-browser-globals-in-ssr-bad"></span>

**错误示例**

setup 立即读取 `window.innerWidth`，但组件在服务器运行时不存在 `window`。

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
```

<span id="ssr-no-browser-globals-in-ssr-good"></span>

**正确示例**

初始宽度是服务端安全的 ref 值，浏览器访问移到 `onMounted`，在客户端运行而不在 SSR setup 期间运行。

```vue annotate="add:2,3,4,5,6"
<script setup lang="ts">
const width = ref(0);

onMounted(() => {
  width.value = window.innerWidth;
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_browser_globals_in_ssr.rs#L158) · [全部规则](all.md)

### `ssr/no-hydration-mismatch`

禁止导致水合不匹配的不确定值

[错误示例](#ssr-no-hydration-mismatch-bad) · [正确示例](#ssr-no-hydration-mismatch-good)

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
        "ssr/no-hydration-mismatch": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="ssr-no-hydration-mismatch-bad"></span>

**错误示例**

模板渲染时执行 `Math.random()`，因此服务端和客户端可能为同一段落生成不同文本。

```vue annotate="remove:2"
<template>
  <p>{{ Math.random() }}</p>
</template>
```

<span id="ssr-no-hydration-mismatch-good"></span>

**正确示例**

段落渲染稳定的 `seed` 状态，不再生成新随机结果。在此 Nuxt 风格示例中，`useState` 提供共享状态，初始值是常量 `"stable"`。

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
const seed = useState("seed", () => "stable");
</script>

<template>
  <p>{{ seed }}</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/ssr/no_hydration_mismatch.rs#L122) · [全部规则](all.md)

### `type/no-floating-promises`

禁止悬空（未处理）的 Promise

[错误示例](#type-no-floating-promises-bad) · [正确示例](#type-no-floating-promises-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**错误示例**

异步 `save` 函数返回 Promise，但独立的 `save()` 调用既不等待也不返回它，也未显式标记有意丢弃。

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**正确示例**

`void save()` 显式标记此规则接受的发出后不等待意图。这是显式丢弃标记，不是拒绝处理器。

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [全部规则](all.md)

### `type/no-reactivity-loss`

禁止赋值和调用中对响应式值取普通快照

[错误示例](#type-no-reactivity-loss-bad) · [正确示例](#type-no-reactivity-loss-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**错误示例**

`const count = state.count` 对响应式属性取普通数字快照，因此之后 `state.count` 的更新不会反映到该绑定。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**正确示例**

`toRef(state, "count")` 使 `count` 继续关联原响应式属性，不复制其当前原始类型值。

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [全部规则](all.md)

### `type/no-unsafe-template-binding`

禁止解析为不安全类型的模板绑定

[错误示例](#type-no-unsafe-template-binding-bad) · [正确示例](#type-no-unsafe-template-binding-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**错误示例**

插值的 `value` 被显式注解为 `any`，检查器无法为模板绑定提供安全的具体类型。

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**正确示例**

将注解改为 `string`，为同一插值提供可检查的具体类型，而不改变渲染值。

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [全部规则](all.md)

### `type/require-typed-emits`

要求 defineEmits 具有类型定义

[错误示例](#type-require-typed-emits-bad) · [正确示例](#type-require-typed-emits-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**错误示例**

仅数组形式的 `defineEmits(["save"])` 声明事件名称，却没有类型化载荷约定。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**正确示例**

`defineEmits<{ save: [] }>()` 以空载荷元组声明带类型的 `save` 事件，明确表示不接收载荷参数。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [全部规则](all.md)

### `type/require-typed-props`

要求 defineProps 具有类型定义

[错误示例](#type-require-typed-props-bad) · [正确示例](#type-require-typed-props-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**错误示例**

仅数组形式的 `defineProps(["title"])` 只按名称声明 `title`，没有提供类型。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**正确示例**

`defineProps<{ title: string }>()` 为 `title` 提供显式字符串类型，替代仅名称的运行时声明。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [全部规则](all.md)

### `type/strict-boolean-expressions`

要求脚本和模板条件使用安全的布尔表达式

[错误示例](#type-strict-boolean-expressions-bad) · [正确示例](#type-strict-boolean-expressions-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 参见[类型化选项和默认值](/rules/options.md)。

显式启用 typeAware 和此规则。默认不允许可空数字，允许非空数字。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**错误示例**

`if (count)` 依赖可空数字绑定的真值性，而非显式布尔检查，也将零与缺失混为一谈。

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**正确示例**

`count !== undefined && count > 0` 分别检查存在性和正值，缩窄可选值后产生显式布尔条件。

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [全部规则](all.md)

### `vapor/no-inline-template`

禁止已弃用的 inline-template 属性

[错误示例](#vapor-no-inline-template-bad) · [正确示例](#vapor-no-inline-template-good)

默认严重程度: `error`  
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
        "vapor/no-inline-template": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-inline-template-bad"></span>

**错误示例**

LegacyCard 为子标记使用 inline-template 属性。

```vue annotate="remove:2,3"
<template>
  <LegacyCard inline-template>
    <p>Profile</p>
  </LegacyCard>
</template>
```

<span id="vapor-no-inline-template-good"></span>

**正确示例**

标记通过默认插槽传入，而非内联模板。

```vue annotate="add:2,3,4,5"
<template>
  <LegacyCard>
    <template #default>
      <p>Profile</p>
    </template>
  </LegacyCard>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/no_inline_template.rs#L31) · [全部规则](all.md)

### `vapor/no-vue-lifecycle-events`

禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）

[错误示例](#vapor-no-vue-lifecycle-events-bad) · [正确示例](#vapor-no-vue-lifecycle-events-good)

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
        "vapor/no-vue-lifecycle-events": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-no-vue-lifecycle-events-bad"></span>

**错误示例**

输入框使用了 @vue:mounted 模板生命周期事件。

```vue annotate="remove:2"
<template>
  <input @vue:mounted="focusInput" />
</template>
```

<span id="vapor-no-vue-lifecycle-events-good"></span>

**正确示例**

onMounted 通过受支持的脚本生命周期钩子访问具名模板引用，并聚焦输入框。

```vue annotate="add:1,2,3,4,5,6,7,8,10"
<script setup lang="ts" vapor>
const input = useTemplateRef<HTMLInputElement>("input");

onMounted(() => {
  input.value?.focus();
});
</script>

<template>
  <input ref="input" />
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/vapor/no_vue_lifecycle_events.rs#L34) · [全部规则](all.md)

### `vapor/prefer-static-class`

字符串字面量优先使用静态 class，而不是动态 class 绑定

[错误示例](#vapor-prefer-static-class-bad) · [正确示例](#vapor-prefer-static-class-good)

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
        "vapor/prefer-static-class": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-prefer-static-class-bad"></span>

**错误示例**

class 绑定对常量字符串求值，但类名并不会改变。

```vue annotate="remove:2"
<template>
  <section :class="'panel panel-primary'">Profile</section>
</template>
```

<span id="vapor-prefer-static-class-good"></span>

**正确示例**

静态 class 属性表达相同的 panel 类名，无需绑定。

```vue annotate="add:2"
<template>
  <section class="panel panel-primary">Profile</section>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/prefer_static_class.rs#L31) · [全部规则](all.md)

### `vapor/require-vapor-attribute`

建议为 script setup 添加 vapor 属性

[错误示例](#vapor-require-vapor-attribute-bad) · [正确示例](#vapor-require-vapor-attribute-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 尚未在 SFC lint 中实现  
适用范围: Vue SFC 模板和代码块，包括规则所需的脚本上下文  
选项: 没有规则专属选项。可以配置严重程度和预设。

当前支持情况: `no-sfc-finding`

此规则是具有空回调的占位项。添加 vapor 会选择 Vapor 编译；当前 linter 不会因缺少该属性而报告此目录 ID。

**可配置的 ID（当前没有 SFC 诊断）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "vapor/require-vapor-attribute": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="vapor-require-vapor-attribute-bad"></span>

**错误示例**

script setup 块缺少 Vapor 编译属性。这是预期规范：当前为空的规则回调不会诊断它。

```vue annotate="remove:1"
<script setup>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

<span id="vapor-require-vapor-attribute-good"></span>

**正确示例**

添加 vapor 会选择 Vapor 编译。这展示预期修复，并不表示当前 linter 会报告此目录规则。

```vue annotate="add:1"
<script setup vapor>
const count = 0;
</script>
<template><p>{{ count }}</p></template>
```

正确示例展示预期规范；当前 SFC 流程不会为任一示例生成这条规则的诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/vapor/require_vapor_attribute.rs#L17) · [全部规则](all.md)

### `vize:croquis/cf/array-mutation`

通过索引修改数组，无法被该响应式数组跟踪。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

仅适用于历史 Vue 2.7：此情形须使用匹配的 Vue 2.7 和 SFC 编译器依赖。Vue 3 代理跟踪数组索引赋值，因此 `items[0] = next` 在 Vue 3 中具有响应性，并非 Vue 3 缺陷。此公开诊断代码目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import Vue from 'vue';
import App from './App.vue';
new Vue({ render: h => h(App) }).$mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script lang="ts">
import Vue from 'vue';
import { replaceFirst } from './replace-first';
export default Vue.extend({
  data() { return { items: ['Before'] }; },
  methods: { replace() { replaceFirst(this.items, 'After'); } },
});
</script>
<template><section><p>{{ items[0] }}</p><button @click="replace">Replace</button></section></template>

```

<span id="vize-croquis-cf-array-mutation-bad"></span>

**错误示例**

在此历史 Vue 2.7 项目中，`items[0] = next` 修改数组却不通知 Vue 2 数组观察器，因此显示的首项不一定更新。

`replace-first.ts`

```ts annotate="remove:2"
export function replaceFirst(items: string[], next: string): void {
  items[0] = next;
}

```

<span id="vize-croquis-cf-array-mutation-good"></span>

**正确示例**

`splice(0, 1, next)` 使用 Vue 2 可观察的数组修改方法，使同样的替换能更新视图。

`replace-first.ts`

```ts annotate="add:2"
export function replaceFirst(items: string[], next: string): void {
  items.splice(0, 1, next);
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/async-boundary`

响应式状态跨越异步边界，可能被观察为过期值。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/async-boundary": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-async-boundary-bad"></span>

**错误示例**

较慢的旧查询可能在新查询之后完成并覆盖 `result`，因为侦听器没有失效清理。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="remove:10,11"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value) => {
    result.value = await load(value);
  },
);
</script>
```

<span id="vize-croquis-cf-async-boundary-good"></span>

**正确示例**

在等待之前注册清理：中止旧请求并将其 `active` 标志设为失效，只赋值仍有效的响应。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="add:10,11,12,13,14,15,16,17,18,19,20"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/async-no-suspense`

异步组件在没有 Suspense 边界的情况下渲染。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

当前支持情况: `no-source-async-fact`

边界诊断实现读取 macros.is_async()，但当前源代码解析将顶层 await 记录在 script-setup 作用域。因此，下方完整的错误与正确源代码对照不会通过当前 CLI 产生 async-no-suspense 诊断。它解释 Suspense 规范；补充缺失宏事实属于后续实现工作。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-async-no-suspense-bad"></span>

**错误示例**

子组件具有顶层 await，但父组件未提供 `<Suspense>` 边界。当前源代码解析未提供发出此诊断代码所需的宏事实。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

<span id="vize-croquis-cf-async-no-suspense-good"></span>

**正确示例**

父组件将同一异步子组件包在 `<Suspense>` 中，并提供加载后备内容。这展示规范；当前分析遍对两种源代码形式都不发出该诊断。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Suspense><Child /><template #fallback><p>Loading</p></template></Suspense></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const greeting = await Promise.resolve("Hello");
</script>
<template><p>{{ greeting }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/browser-api-ssr`

在组件可能于服务端渲染的位置使用了浏览器专用 API。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/browser-api-ssr": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-browser-api-ssr-bad"></span>

**错误示例**

`window.innerWidth` 在 setup 期间运行，但 SSR 环境没有浏览器 `window`。

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const width = window.innerWidth;
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-browser-api-ssr-good"></span>

**正确示例**

将 ref 初始化为服务端安全的值，并在客户端挂载后运行的 `onMounted` 中读取 `window`。

`App.vue`

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { onMounted, ref } from "vue";
const width = ref(0);
onMounted(() => { width.value = window.innerWidth; });
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/circular-dep`

组件相互导入，形成循环。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

此示例展示具体的立即初始化循环。递归 Vue 组件或任意循环导入并不自动构成错误。目前没有实现发出此诊断约定代码。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { aLabel } from './a';
</script>

<template>
<p>{{ aLabel }}</p>
</template>

```

`labels.ts`

```ts
export const aPrefix = 'A';
export const bPrefix = 'B';

```

<span id="vize-croquis-cf-circular-dep-bad"></span>

**错误示例**

`a.ts` 导入 `b.ts`，后者又导入 `a.ts`。两者都立即读取另一模块尚未初始化的常量来初始化自身常量，导致暂时性死区错误。

`a.ts`

```ts annotate="remove:1,2"
import { bLabel } from './b';
export const aLabel = 'A' + bLabel;

```

`b.ts`

```ts annotate="remove:1,2"
import { aLabel } from './a';
export const bLabel = 'B' + aLabel;

```

<span id="vize-croquis-cf-circular-dep-good"></span>

**正确示例**

两个模块都从独立的 `labels.ts` 读取已初始化前缀，移除循环及立即发生的交叉读取。

`a.ts`

```ts annotate="add:1,2"
import { bPrefix } from './labels';
export const aLabel = 'A' + bPrefix;

```

`b.ts`

```ts annotate="add:1,2"
import { aPrefix } from './labels';
export const bLabel = 'B' + aPrefix;

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/circular-reactive-dependency`

响应式计算相互依赖，形成循环。

默认严重程度: 依上下文而定  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/circular-reactive-dependency": "warn" },
    },
  },
});
```

```sh
vp run lint
```

示例验证状态: `illustrative-source-pair`

下方完整 Vue 项目展示更新反馈及其修复，但不构成已验证的 CLI 诊断复现：诊断实现需要附图所示的保留响应式流引用身份和边。这些源代码不能证明当前源代码路径会发出此确切代码。专门的跟踪 ID 图诊断对照仍与源代码语法检查分开。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`count-key.ts`

```ts
import type { InjectionKey, Ref } from 'vue';
export const countKey: InjectionKey<Ref<number>> = Symbol('count');
```

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from 'vue';
import { countKey } from './count-key';
import CycleView from './CycleView.vue';
const count = ref(1); // A: the provider-owned source.
provide(countKey, count);
</script>
<template>
  <button @click="count++">Increment</button>
  <CycleView />
</template>
```

<span id="vize-croquis-cf-circular-reactive-dependency-bad"></span>

**错误示例**

App 拥有并提供 count（A）。CycleView 派生 nextCount（B），随后立即将每个派生值写回同一注入的 count。每次写入再次改变计算输入，形成 A → B → A 更新反馈。下方保留图中的身份代表这两个引用，而非仅名称相同的无关绑定。

`CycleView.vue`

```vue annotate="remove:2,6"
<script setup lang="ts">
import { computed, inject, watch } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
watch(nextCount, value => { count.value = value; }, { immediate: true });
</script>
<template><p>{{ nextCount }}</p></template>
```

```text annotate="remove:2"
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B; B -> A
```

<span id="vize-croquis-cf-circular-reactive-dependency-good"></span>

**正确示例**

移除将 B 写回 A 的侦听器。App 保留 count 的所有权，仅通过显式 Increment 操作改变它；CycleView 读取派生 nextCount，不将结果反馈回去。同样的引用只保留 A → B 依赖。

`CycleView.vue`

```vue annotate="add:2"
<script setup lang="ts">
import { computed, inject } from 'vue';
import { countKey } from './count-key';
const count = inject(countKey)!; // App provides this same A reference.
const nextCount = computed(() => count.value + 1); // B: the derived consumer.
</script>
<template><p>{{ nextCount }}</p></template>
```

```text annotate="add:2"
Tracked references: A = provider source; B = consumer reference
Tracked flows: A -> B
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/closure-captures-reactive`

闭包捕获了响应式值，无法看到后续更新。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { computed, ref } from 'vue';
import { makeReader } from './reader';
const count = ref(0);
const read = makeReader(count);
const shown = computed(read);
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ shown }}</p>
</template>

```

<span id="vize-croquis-cf-closure-captures-reactive-bad"></span>

**错误示例**

`makeReader` 在创建闭包前复制 `count.value`。计算读取器随后返回初始数字，没有读取响应式依赖。

`reader.ts`

```ts annotate="remove:3,4"
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  const captured = count.value;
  return () => captured;
}

```

<span id="vize-croquis-cf-closure-captures-reactive-good"></span>

**正确示例**

闭包在调用时读取 `count.value`，使计算 getter 能跟踪 ref，并在递增后更新 `shown`。

`reader.ts`

```ts annotate="add:3"
import type { Ref } from 'vue';
export function makeReader(count: Ref<number>): () => number {
  return () => count.value;
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/composable-outside-setup`

组合式函数在 `setup` 外被调用。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

问题针对这个依赖生命周期的组合式函数，并非全面禁止普通工具函数或 setup 外的所有 Composition API 调用。此诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useTitle } from './use-title';
const title = useTitle();
</script>

<template>
<h1>{{ title }}</h1>
</template>

```

<span id="vize-croquis-cf-composable-outside-setup-bad"></span>

**错误示例**

导入 `use-title.ts` 时，在组件 setup 激活前注册了 `onMounted`。稍后调用导出函数仅返回模块级 ref，无法补救错失的生命周期所有权。

`use-title.ts`

```ts annotate="remove:2,3,4"
import { onMounted, ref } from 'vue';
const title = ref('Before mount');
onMounted(() => { title.value = 'Mounted'; });
export function useTitle() { return title; }

```

<span id="vize-croquis-cf-composable-outside-setup-good"></span>

**正确示例**

状态创建和钩子注册都移到 `useTitle`，由 App 在 setup 中同步调用。挂载钩子现在属于该 App 实例。

`use-title.ts`

```ts annotate="add:2,3,4,5,6"
import { onMounted, ref } from 'vue';
export function useTitle() {
  const title = ref('Before mount');
  onMounted(() => { title.value = 'Mounted'; });
  return title;
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/computed-side-effects`

计算 getter 写入状态或执行其他副作用。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled, lastCalculated } = useDouble();
</script>

<template>
<button @click="count++">Increment {{ count }}</button><p>{{ doubled }} / {{ lastCalculated }}</p>
</template>

```

<span id="vize-croquis-cf-computed-side-effects-bad"></span>

**错误示例**

求值 `doubled` 会写入 `lastCalculated`，因此读取计算值也会修改独立状态，将副作用与惰性 getter 的读取时机耦合。

`use-double.ts`

```ts annotate="remove:1,5,6,7,8,9"
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => {
    const next = count.value * 2;
    lastCalculated.value = next;
    return next;
  });
  return { count, doubled, lastCalculated };
}

```

<span id="vize-croquis-cf-computed-side-effects-good"></span>

**正确示例**

getter 只返回派生数字。独立侦听器负责在 `count` 变化时写入 `lastCalculated`，也包括初始值。

`use-double.ts`

```ts annotate="add:1,5,6"
import { computed, ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const lastCalculated = ref(0);
  const doubled = computed(() => count.value * 2);
  watch(count, next => { lastCalculated.value = next * 2; }, { immediate: true });
  return { count, doubled, lastCalculated };
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/deep-import`

导入链超过项目允许的深度。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

这是显式选择的项目布局策略，并未宣称存在受支持的深度阈值或选项。此诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { label } from './entry';
</script>

<template>
<p>{{ label }}</p>
</template>

```

`value.ts`

```ts
export const label = 'Notice';

```

`level-one.ts`

```ts
export { label } from './level-two';

```

`level-two.ts`

```ts
export { label } from './level-three';

```

`level-three.ts`

```ts
export { label } from './value';

```

`public-api.ts`

```ts
export { label } from './value';

```

<span id="vize-croquis-cf-deep-import-bad"></span>

**错误示例**

入口让简单值经过 `level-one`、`level-two` 和 `level-three`，对期望浅层公共边界的项目形成不必要的深导入链。

`entry.ts`

```ts annotate="remove:1"
export { label } from './level-one';

```

<span id="vize-croquis-cf-deep-import-good"></span>

**正确示例**

入口使用直接重新导出该值的 `public-api.ts`。消费者保留相同导入名称，导入链变短。

`entry.ts`

```ts annotate="add:1"
export { label } from './public-api';

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/destructuring-breaks-reactivity`

解构响应式对象会复制字段并丢失跟踪。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/destructuring-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-bad"></span>

**错误示例**

普通解构 `props` 对象会复制当前 `item` 值；这与 Vue 3.5 直接解构 `defineProps()` 不同。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps<{ item: { name: string } }>();
const { item } = props;
</script>
```

<span id="vize-croquis-cf-destructuring-breaks-reactivity-good"></span>

**正确示例**

`toRef(props, "item")` 保留与 `props` 属性的联系。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:2,3,5"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ item: { name: string } }>();
const item = toRef(props, "item");
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/di-outside-setup`

`provide` 或 `inject` 在 `setup` 外被调用。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

此示例使用组件 provide/inject。`app.provide` 和受支持的 `app.runWithContext` 注入是不同且有效的所有权接口，不受此情形禁止。目前没有实现发出此诊断约定代码。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`theme.ts`

```ts
import { inject, provide } from 'vue';
import type { InjectionKey } from 'vue';
export const ThemeKey: InjectionKey<string> = Symbol('theme');
export function provideTheme() { provide(ThemeKey, 'dark'); }
export function useTheme() { return inject(ThemeKey, 'light'); }

```

`ThemedText.vue`

```vue
<script setup lang="ts">
import { useTheme } from './theme';
const theme = useTheme();
</script>

<template>
<p>{{ theme }}</p>
</template>

```

<span id="vize-croquis-cf-di-outside-setup-bad"></span>

**错误示例**

`main.ts` 在没有激活组件实例时调用组件 `provide`。子组件的 `inject` 无法获取预期祖先值，因此使用 `light`。

`main.ts`

```ts annotate="remove:1,2,3,4,5,6"
import { createApp } from 'vue';
import App from './App.vue';
import { provideTheme } from './theme';
provideTheme();
createApp(App).mount('#app');

```

`App.vue`

```vue
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
</script>

<template>
<ThemedText />
</template>

```

<span id="vize-croquis-cf-di-outside-setup-good"></span>

**正确示例**

App 在渲染子组件前从 setup 调用提供者。子组件现在从组件祖先继承 `dark`。

`App.vue`

```vue annotate="add:3,4"
<script setup lang="ts">
import ThemedText from './ThemedText.vue';
import { provideTheme } from './theme';
provideTheme();
</script>

<template>
<ThemedText />
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/dom-access-without-next-tick`

在 Vue 完成更新刷新之前读取 DOM。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`read-label.ts`

```ts
export function readLabel(node: HTMLElement | null): string {
  return node?.textContent ?? '';
}

```

<span id="vize-croquis-cf-dom-access-without-next-tick-bad"></span>

**错误示例**

点击处理器递增 `count` 后立即读取渲染段落，此时 Vue 尚未刷新计划中的 DOM 更新。`sampled` 可能包含之前的计数。

`App.vue`

```vue annotate="remove:2,7"
<script setup lang="ts">
import { ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
function increment() {
  count.value++;
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

<span id="vize-croquis-cf-dom-access-without-next-tick-good"></span>

**正确示例**

写入状态后等待 `nextTick()`，让 Vue 先更新段落，再由 `readLabel` 读取文本。

`App.vue`

```vue annotate="add:2,7,9"
<script setup lang="ts">
import { nextTick, ref } from 'vue';
import { readLabel } from './read-label';
const count = ref(0);
const label = ref<HTMLElement | null>(null);
const sampled = ref('');
async function increment() {
  count.value++;
  await nextTick();
  sampled.value = readLabel(label.value);
}
</script>

<template>
<button @click="increment">Increment</button><p ref="label">{{ count }}</p><p>DOM sample: {{ sampled }}</p>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/duplicate-id`

多个组件使用同一个元素 id。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/duplicate-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./CheckoutForm.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-duplicate-id-bad"></span>

**错误示例**

可达的配送与账单组件都渲染 `id="postal-code"`，使其标签共享不明确的文档目标。

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue annotate="remove:2,3"
<template>
  <label for="postal-code">Shipping postal code</label>
  <input id="postal-code" />
</template>
```

`BillingAddress.vue`

```vue annotate="remove:2,3"
<template>
  <label for="postal-code">Billing postal code</label>
  <input id="postal-code" />
</template>
```

<span id="vize-croquis-cf-duplicate-id-good"></span>

**正确示例**

每个组件调用 `useId()`，将自身的值同时绑定给标签和输入框，在不重复字面量 ID 的情况下保留关联。

`CheckoutForm.vue`

```vue
<script setup lang="ts">
import BillingAddress from "./BillingAddress.vue";
import ShippingAddress from "./ShippingAddress.vue";
</script>

<template>
  <ShippingAddress />
  <BillingAddress />
</template>
```

`ShippingAddress.vue`

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Shipping postal code</label>
  <input :id="postalCodeId" />
</template>
```

`BillingAddress.vue`

```vue annotate="add:1,2,3,4,5,6,8,9"
<script setup lang="ts">
import { useId } from "vue";

const postalCodeId = useId();
</script>

<template>
  <label :for="postalCodeId">Billing postal code</label>
  <input :id="postalCodeId" />
</template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/event-listener-leak`

注册了事件监听器，却从未移除。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useWidth } from './use-width';
const width = useWidth();
</script>

<template>
<p>{{ width }}</p>
</template>

```

<span id="vize-croquis-cf-event-listener-leak-bad"></span>

**错误示例**

挂载时添加的 window resize 监听器捕获组件 width ref，但卸载时从不移除。重复挂载可能保留未使用的监听器和状态。

`use-width.ts`

```ts annotate="remove:1"
import { onMounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  return width;
}

```

<span id="vize-croquis-cf-event-listener-leak-good"></span>

**正确示例**

`onUnmounted` 移除挂载时注册的同一个 `resize` 函数，结束该实例外部监听器的生命周期。

`use-width.ts`

```ts annotate="add:1,6"
import { onMounted, onUnmounted, ref } from 'vue';
export function useWidth() {
  const width = ref(0);
  const resize = () => { width.value = window.innerWidth; };
  onMounted(() => { resize(); window.addEventListener('resize', resize); });
  onUnmounted(() => { window.removeEventListener('resize', resize); });
  return width;
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/event-modifier`

事件监听器使用了发出事件不支持的修饰符。

默认严重程度: info  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-event-modifier-bad"></span>

**错误示例**

`.stop` 假定子组件自定义 `save` 事件具有原生事件的传播方法，但其载荷不一定是 DOM Event。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save.stop="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-event-modifier-good"></span>

**正确示例**

从自定义事件监听器移除 `.stop`；需要时在实际 DOM 监听器处理原生传播。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/hydration-risk`

此诊断代码归类多种响应性问题，包括将 prop 复制到 ref。它并不表示跨文件分析遍会检测所有 Date.now() 表达式。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/hydration-risk": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-hydration-risk-bad"></span>

**错误示例**

子组件仅初始化一次 `ref(props.count)`，因此局部 count 不再跟随后续父组件 prop 变化。这是当前的 prop 到 ref 诊断产生逻辑，不是普遍的不确定 SSR 示例。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue annotate="remove:2,4"
<script setup lang="ts">
import { ref } from "vue";
const props = defineProps<{ count: number }>();
const count = ref(props.count);
</script>
<template><p>{{ count }}</p></template>
```

<span id="vize-croquis-cf-hydration-risk-good"></span>

**正确示例**

`toRef(props, "count")` 指向 prop，不再将初始值复制到独立状态。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :count="0" /></template>
```

`Child.vue`

```vue annotate="add:2,4"
<script setup lang="ts">
import { toRef } from "vue";
const props = defineProps<{ count: number }>();
const count = toRef(props, "count");
</script>
<template><p>{{ count }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/inherit-attrs-unused`

设置了 `inheritAttrs: false`，但组件从不读取属性。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-inherit-attrs-unused-bad"></span>

**错误示例**

子组件设置 `inheritAttrs: false`，却从不转发父组件的 `class="notice"` 属性。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="remove:4"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main>Content</main></template>
```

<span id="vize-croquis-cf-inherit-attrs-unused-good"></span>

**正确示例**

保留显式继承控制，将 `$attrs` 绑定到预期的 `<main>` 目标。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="add:4"
<script setup lang="ts">
defineOptions({ inheritAttrs: false });
</script>
<template><main v-bind="$attrs">Content</main></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/inject-without-symbol`

`inject` 使用普通键，而不是 `InjectionKey` symbol。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/inject-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-inject-without-symbol-bad"></span>

**错误示例**

消费者注入无类型字符串键 `"theme"`，没有与提供者共享的 symbol 身份。

`ThemeProvider.vue`

```vue annotate="remove:6"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-inject-without-symbol-good"></span>

**正确示例**

消费者和提供者导入同一个 `ThemeKey`，不重复字符串名称。

`ThemeProvider.vue`

```vue annotate="add:4,7"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="add:3,5"
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/injected-async-mutation-race`

注入值被可能发生竞态的异步任务修改。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/injected-async-mutation-race": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./StoreProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export async function loadCount(query: string, options?: { signal?: AbortSignal }): Promise<number> {
  const response = await fetch(`/count?q=${encodeURIComponent(query)}`, options);
  return Number(await response.text());
}
```

`CountSummary.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { StoreKey } from "./keys/store";
const store = inject(StoreKey)!;
</script>
<template><p>{{ store.count }}</p></template>
```

<span id="vize-croquis-cf-injected-async-mutation-race-bad"></span>

**错误示例**

`CountLoader.vue` 将等待得到的结果直接写入与 `CountSummary.vue` 共享的注入 store，使过期任务影响两个消费者。

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue annotate="remove:12"
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);
</script>

<template>
  <CountLoader />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue annotate="remove:3,4,6,9,10"
<script setup lang="ts">
import { loadCount } from "./api";
import { inject, ref, watch } from "vue";
import { StoreKey } from "./keys/store";

const store = inject(StoreKey)!;
const query = ref("");

watch(query, async (value) => {
  store.count = await loadCount(value);
});
</script>
```

<span id="vize-croquis-cf-injected-async-mutation-race-good"></span>

**正确示例**

加载器取消失效任务，只发出有效结果。提供者通过 `applyLoadedCount` 拥有 store 修改权。

`keys/store.ts`

```ts
import type { InjectionKey } from "vue";

export interface Store {
  count: number;
}

export const StoreKey: InjectionKey<Store> = Symbol("store");
```

`StoreProvider.vue`

```vue annotate="add:9,10,11,12,16"
<script setup lang="ts">
import { provide, reactive } from "vue";
import CountLoader from "./CountLoader.vue";
import CountSummary from "./CountSummary.vue";
import { StoreKey, type Store } from "./keys/store";

const store = reactive<Store>({ count: 0 });
provide(StoreKey, store);

function applyLoadedCount(count: number) {
  store.count = count;
}
</script>

<template>
  <CountLoader @loaded="applyLoadedCount" />
  <CountSummary />
</template>
```

`CountLoader.vue`

```vue annotate="add:3,5,8,9,10,11,12,13,14,15,16,17,18"
<script setup lang="ts">
import { loadCount } from "./api";
import { ref, watch } from "vue";

const emit = defineEmits<{ loaded: [count: number] }>();
const query = ref("");

watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;

  onCleanup(() => {
    active = false;
    controller.abort();
  });

  const count = await loadCount(value, { signal: controller.signal });
  if (active) emit("loaded", count);
});
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/lifecycle-outside-setup`

生命周期钩子在 `setup` 外注册。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`install-title.ts`

```ts
import { onMounted } from 'vue';
export function installTitle() {
  onMounted(() => { document.title = 'Mounted application'; });
}

```

<span id="vize-croquis-cf-lifecycle-outside-setup-bad"></span>

**错误示例**

入口在挂载应用前调用 `installTitle()`，因此 `onMounted` 在没有激活组件 setup 上下文时注册。

`main.ts`

```ts annotate="remove:1,2,3,4,5,6"
import { createApp } from 'vue';
import App from './App.vue';
import { installTitle } from './install-title';
installTitle();
createApp(App).mount('#app');

```

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">

</script>

<template>
<p>Application</p>
</template>

```

<span id="vize-croquis-cf-lifecycle-outside-setup-good"></span>

**正确示例**

从 App 的 setup 同步调用同一辅助函数，将生命周期回调关联到该实例的挂载。

`App.vue`

```vue annotate="add:2,3"
<script setup lang="ts">
import { installTitle } from './install-title';
installTitle();
</script>

<template>
<p>Application</p>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/lifecycle-without-cleanup`

生命周期钩子启动任务，却从不清理。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-bad"></span>

**错误示例**

挂载时注册 window resize 监听器，卸载时却不移除同一回调。

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
import { onMounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-lifecycle-without-cleanup-good"></span>

**正确示例**

`onUnmounted` 使用与 `addEventListener` 相同的事件名称和函数身份移除监听器。

`App.vue`

```vue annotate="add:2,5"
<script setup lang="ts">
import { onMounted, onUnmounted } from "vue";
const resize = () => {};
onMounted(() => { window.addEventListener("resize", resize); });
onUnmounted(() => { window.removeEventListener("resize", resize); });
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/missing-required-prop`

未传入必需的 prop。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-missing-required-prop-bad"></span>

**错误示例**

父组件渲染 `<Child />`，没有提供子组件必需的 `title: string` prop。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-missing-required-prop-good"></span>

**正确示例**

`title="Hello"` 提供已解析子组件声明的必需 prop。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/missing-suspense`

异步依赖在 Suspense 边界之外使用。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-missing-suspense-bad"></span>

**错误示例**

`AsyncCard` 具有顶层 await，使其 setup 异步，但 App 没有使用 Suspense 边界协调该依赖。

`App.vue`

```vue annotate="remove:6"
<script setup lang="ts">
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<AsyncCard />
</template>

```

<span id="vize-croquis-cf-missing-suspense-good"></span>

**正确示例**

App 将异步子组件包在 `Suspense` 内，在子组件 setup 完成前提供加载后备内容。

`App.vue`

```vue annotate="add:2,7"
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/module-scope-reactive`

响应式状态在模块作用域创建，被所有调用者共享。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

有意共享的应用 store 在模块作用域创建响应式状态是合法的。此示例假定组件与请求隔离；此公开诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { createCounter } from './counter';
const { count } = createCounter();
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-module-scope-reactive-bad"></span>

**错误示例**

模块仅初始化一次 `count`，两个 Counter 实例收到同一个 ref。虽然示例期望独立实例状态，点击一个却会改变两个计数器。

`counter.ts`

```ts annotate="remove:2,3"
import { ref } from 'vue';
const count = ref(0);
export function createCounter() { return { count }; }

```

<span id="vize-croquis-cf-module-scope-reactive-good"></span>

**正确示例**

在 `createCounter` 内创建 ref，为每个同步 setup 调用提供独立状态对象，使每个按钮拥有自己的计数器。

`counter.ts`

```ts annotate="add:2,3,4,5"
import { ref } from 'vue';
export function createCounter() {
  const count = ref(0);
  return { count };
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/multi-root-attrs`

多根节点组件接收了属性，却没有放置目标。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-multi-root-attrs-bad"></span>

**错误示例**

子组件具有 `<main>` 和 `<aside>` 两个根节点，Vue 没有可自动接收父组件 class 的唯一根节点。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="remove:1"
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-multi-root-attrs-good"></span>

**正确示例**

显式将 `$attrs` 转发到 `<main>`，同时保留第二个根节点。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="add:1"
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/mutated-after-escape`

响应式对象逸出其所有者后被修改。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

这是显式的不可变历史所有权策略，并非普遍禁止传递或随后修改响应式对象。目前没有实现发出此诊断约定。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`archive.ts`

```ts
export interface Profile { name: string }
const records: Readonly<Profile>[] = [];
export function publish(profile: Readonly<Profile>): void { records.push(profile); }
export function latestName(): string { return records.at(-1)?.name ?? ''; }

```

`App.vue`

```vue
<script setup lang="ts">
import { publishProfile } from './profile';
import { latestName } from './archive';
publishProfile();
const archivedName = latestName();
</script>

<template>
<p>Archived name: {{ archivedName }}</p>
</template>

```

<span id="vize-croquis-cf-mutated-after-escape-bad"></span>

**错误示例**

归档保留传给 `publish` 的同一对象，所有者随后修改其名称，使本应属于历史的记录也变成 Grace。TypeScript 的 Readonly 参数不会复制对象。

`profile.ts`

```ts annotate="remove:5"
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish(profile);
  profile.name = 'Grace';
}

```

<span id="vize-croquis-cf-mutated-after-escape-good"></span>

**正确示例**

发布普通副本，将归档的 Ada 记录与之后对响应式 profile 的编辑分开，维护归档快照策略。

`profile.ts`

```ts annotate="add:5"
import { reactive } from 'vue';
import { publish } from './archive';
export function publishProfile(): void {
  const profile = reactive({ name: 'Ada' });
  publish({ ...profile });
  profile.name = 'Grace';
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/non-reactive-provide`

提供的值不具响应性，后代无法看到更新。

默认严重程度: 依上下文而定  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-reactive-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-reactive-provide-bad"></span>

**错误示例**

`ThemeProvider.vue` 提供普通对象。修改其字段不会为注入消费者建立 Vue 响应式依赖。

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue annotate="remove:2,6"
<script setup lang="ts">
import { provide } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = { color: "blue" };
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-non-reactive-provide-good"></span>

**正确示例**

提供者将 theme 包装在 `ref` 内；同一个注入引用可以跟踪后续变化。

`keys/theme.ts`

```ts
export const ThemeKey = Symbol("theme");
```

`ThemeProvider.vue`

```vue annotate="add:2,6"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey } from "./keys/theme";

const theme = ref({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/cross_file_reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/non-unique-id`

循环内的元素 id 对每个列表项并不唯一。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/non-unique-id": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ResultsList.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-non-unique-id-bad"></span>

**错误示例**

每次 `v-for` 迭代都渲染相同的 `result-title` 字面量 ID；循环的 key 不会让 DOM ID 唯一。

`ResultsList.vue`

```vue annotate="remove:6"
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 id="result-title">{{ result.title }}</h2>
  </article>
</template>
```

<span id="vize-croquis-cf-non-unique-id-good"></span>

**正确示例**

标题 ID 包含结果的稳定 ID，为每个列表项产生不同的文档标识符。

`ResultsList.vue`

```vue annotate="add:6"
<script setup lang="ts">
const results = [{ id: "first", title: "First result" }, { id: "second", title: "Second result" }];
</script>
<template>
  <article v-for="result in results" :key="result.id">
    <h2 :id="`result-${result.id}-title`">{{ result.title }}</h2>
  </article>
</template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/element_id.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/object-identity-comparison`

响应式对象按身份比较，解包后身份会改变。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

此示例假定 ID 唯一标识记录。比较指向同一响应式代理的两个引用仍然有效；此诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`user.ts`

```ts
import { reactive } from 'vue';
export function makeUser() {
  const raw = { id: 7, name: 'Ada' };
  return { raw, proxy: reactive(raw) };
}

```

<span id="vize-croquis-cf-object-identity-comparison-bad"></span>

**错误示例**

`proxy === raw` 比较包装对象身份，因此即使二者代表同一用户记录，结果也是 false。应用希望比较记录身份，而非包装对象身份。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy === raw;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

<span id="vize-croquis-cf-object-identity-comparison-good"></span>

**正确示例**

比较稳定的记录 `id`，回答预期问题，不依赖对象是原始对象还是代理。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import { makeUser } from './user';
const { raw, proxy } = makeUser();
const sameRecord = proxy.id === raw.id;
</script>

<template>
<p>Same record: {{ sameRecord }}</p>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/pinia-getter`

Pinia getter 未通过 `storeToRefs` 读取，无法保持响应性。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

必须安装 Pinia，且 main.ts 须在挂载前安装其插件。在受跟踪的计算或模板内直接读取 `store.doubled` 是有效的；此处问题是取普通快照。该诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import { createPinia } from 'pinia';
import App from './App.vue';
createApp(App).use(createPinia()).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`counter-store.ts`

```ts
import { defineStore } from 'pinia';
export const useCounterStore = defineStore('counter', {
  state: () => ({ count: 0 }),
  getters: { doubled: state => state.count * 2 },
});

```

<span id="vize-croquis-cf-pinia-getter-bad"></span>

**错误示例**

`const doubled = store.doubled` 在 setup 时复制 getter 的当前数字。复制的数字不会跟随后续 `store.count` 更新。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const doubled = store.doubled;
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-pinia-getter-good"></span>

**正确示例**

`storeToRefs(store)` 提供响应式 getter ref，可被解构并由模板解包，同时继续关联 store。

`App.vue`

```vue annotate="add:2,5"
<script setup lang="ts">
import { storeToRefs } from 'pinia';
import { useCounterStore } from './counter-store';
const store = useCounterStore();
const { doubled } = storeToRefs(store);
</script>

<template>
<button @click="store.count++">{{ store.count }}</button><p>{{ doubled }}</p>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/prop-type-mismatch`

传入的 prop 值与声明类型不匹配。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-prop-type-mismatch-bad"></span>

**错误示例**

父组件将数字表达式 `42` 传给已解析子组件的 `title: string` prop。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child :title="42" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-prop-type-mismatch-good"></span>

**正确示例**

字面量 `title="Hello"` 提供符合子组件声明的字符串。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/provide-inject-type`

提供值与其 inject 类型不一致。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-inject-type": "warn" },
    },
  },
});
```

```sh
vp run lint
```

此检查比较提供者与消费者的显式类型注解，而非推断的字面量值类型。保留示例中提供者的 `as string` 注解。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-inject-type-bad"></span>

**错误示例**

提供者将 `title` 显式注解为 `string`，后代却为同一键请求 `inject<number>`。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="remove:3"
<script setup lang="ts">
import { inject } from "vue";
const title = inject<number>("title");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-provide-inject-type-good"></span>

**正确示例**

消费者显式的 `inject<string>` 与提供者注解一致。保留 `as string`：此诊断产生逻辑比较显式注解，而非推断的字面量类型。

`App.vue`

```vue
<script setup lang="ts">
import { provide } from "vue";
import Child from "./Child.vue";
provide("title", "Hello" as string);
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="add:3"
<script setup lang="ts">
import { inject } from "vue";
const title = inject<string>("title");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/provide-without-symbol`

`provide` 使用普通键，而不是 `InjectionKey` symbol。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/provide-without-symbol": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./ThemeProvider.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-provide-without-symbol-bad"></span>

**错误示例**

两个组件都使用字符串 `"theme"`，无关功能可能意外复用此键。

`ThemeProvider.vue`

```vue annotate="remove:5,6"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";

const theme = ref({ color: "blue" });
provide("theme", theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import { inject } from "vue";

const theme = inject("theme");
</script>
```

<span id="vize-croquis-cf-provide-without-symbol-good"></span>

**正确示例**

导出一个带类型的 `ThemeKey` symbol，并在 provide 和 inject 处导入同一值。创建描述相同但独立的 symbols 并不能建立联系。

`ThemeProvider.vue`

```vue annotate="add:4,6,7"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="add:3,5"
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

`keys/theme.ts`

```ts annotate="add:1,2,3,4,5,6,7"
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/keys.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/reactive-export`

响应式状态从模块导出。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

有意共享的应用 store 可以导出响应式状态。此情形要求状态隔离，并未声称所有响应式导出都无效。目前没有实现发出此诊断约定。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

<span id="vize-croquis-cf-reactive-export-bad"></span>

**错误示例**

模块导出一个已初始化的响应式对象，所有导入者都收到同一个 count。在请求间共享的 SSR 模块中，这违背示例要求的实例与请求状态隔离。

`state.ts`

```ts annotate="remove:2"
import { reactive } from 'vue';
export const state = reactive({ count: 0 });

```

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
import { state } from './state';
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

<span id="vize-croquis-cf-reactive-export-good"></span>

**正确示例**

模块导出工厂，由 App 在 setup 内调用。每个实例获得新的响应式 count，而非导出的单例。

`state.ts`

```ts annotate="add:2"
import { reactive } from 'vue';
export function createState() { return reactive({ count: 0 }); }

```

`App.vue`

```vue annotate="add:2,3"
<script setup lang="ts">
import { createState } from './state';
const state = createState();
</script>

<template>
<button @click="state.count++">{{ state.count }}</button>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/reactivity-outside-setup`

响应式 API 在 `setup` 外被调用。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

Vue 允许在组件 setup 外使用 ref/reactive/computed。此处风险是在显式实例隔离策略下产生非预期所有权与共享，并非 API 不合法。此诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Counter from './Counter.vue';
</script>

<template>
<Counter /><Counter />
</template>

```

`Counter.vue`

```vue
<script setup lang="ts">
import { useCounter } from './use-counter';
const { count, doubled } = useCounter();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-reactivity-outside-setup-bad"></span>

**错误示例**

两个响应式 API 都在模块加载时运行，因此两个 Counter 实例共享一个 ref 和计算值，违背独立计数器的预期。

`use-counter.ts`

```ts annotate="remove:2,3,4"
import { computed, ref } from 'vue';
const count = ref(0);
const doubled = computed(() => count.value * 2);
export function useCounter() { return { count, doubled }; }

```

<span id="vize-croquis-cf-reactivity-outside-setup-good"></span>

**正确示例**

`useCounter` 在每次组件 setup 调用中同步创建 ref 和计算值，为每个控件提供自己的状态及受跟踪的派生值。

`use-counter.ts`

```ts annotate="add:2,3,4,5,6"
import { computed, ref } from 'vue';
export function useCounter() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/reassignment-breaks-reactivity`

重新赋值响应式绑定，将其替换为普通值。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/reassignment-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-bad"></span>

**错误示例**

子组件创建 prop ref 后用 `props.user` 覆盖变量，丢弃该 ref 联系。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:5,6,7"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
let user = toRef(props, "user");

user = props.user;
</script>
```

<span id="vize-croquis-cf-reassignment-breaks-reactivity-good"></span>

**正确示例**

将 `toRef` 保留在 `const` 绑定中，移除替换它的重新赋值。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:5"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string } }>();
const user = toRef(props, "user");
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/reference-escapes-scope`

响应式引用逸出拥有其生命周期的作用域。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

Refs 可以合法地从组合式函数返回或跨作用域共享。此示例显式要求快照缓存，并未声称卸载会使 ref 失效。目前没有实现发出此诊断约定。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`saved.ts`

```ts
import type { Ref } from 'vue';
let saved: Ref<number> | number | undefined;
export function remember(value: Ref<number> | number): void { saved = value; }
export function remembered(): Ref<number> | number | undefined { return saved; }

```

<span id="vize-croquis-cf-reference-escapes-scope-bad"></span>

**错误示例**

进程级缓存保留组件的活 count ref，可能让实例状态在卸载后仍可达，并观察后续编辑，但此缓存原本用于存储快照。

`App.vue`

```vue annotate="remove:5"
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

<span id="vize-croquis-cf-reference-escapes-scope-good"></span>

**正确示例**

缓存接收当前普通数字，保留快照而不保留组件拥有的 ref。

`App.vue`

```vue annotate="add:5"
<script setup lang="ts">
import { ref } from 'vue';
import { remember } from './saved';
const count = ref(0);
remember(count.value);
</script>

<template>
<button @click="count++">{{ count }}</button>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/setup-context-violation`

setup 上下文被以 Vue 不允许的方式使用。

默认严重程度: 依上下文而定  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-setup-context-violation-bad"></span>

**错误示例**

`ref(0)` 在普通脚本模块作用域创建，位于此分析器情形所表示的按实例 setup 上下文之外。

`App.vue`

```vue annotate="remove:1,4,6"
<script lang="ts">
import { ref } from "vue";
const count = ref(0);
export default {};
</script>
<template><p>Count</p></template>
```

<span id="vize-croquis-cf-setup-context-violation-good"></span>

**正确示例**

将绑定移入 script setup，使每个组件实例拥有自己的 count，模板也能读取它。

`App.vue`

```vue annotate="add:1,5"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
</script>
<template><p>{{ count }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/setup_context.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/shallow-deep-access`

读取 `shallowReactive` 或 `shallowRef` 的深层属性，却假定它受到跟踪。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.user.name }}</p><button @click="profile.user.name = 'Grace'">Rename</button>
</template>

```

<span id="vize-croquis-cf-shallow-deep-access-bad"></span>

**错误示例**

`shallowReactive` 跟踪根 `user` 属性，但嵌套对象仍是原始对象。修改 `profile.user.name` 不会作为受跟踪的深层修改通知模板。

`profile.ts`

```ts annotate="remove:1,2"
import { shallowReactive } from 'vue';
export function makeProfile() { return shallowReactive({ user: { name: 'Ada' } }); }

```

<span id="vize-croquis-cf-shallow-deep-access-good"></span>

**正确示例**

深层 `reactive` 包装嵌套 user 对象，使同样的名称赋值能触发显示名称更新。

`profile.ts`

```ts annotate="add:1,2"
import { reactive } from 'vue';
export function makeProfile() { return reactive({ user: { name: 'Ada' } }); }

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/spread-breaks-reactivity`

展开响应式对象会复制值并丢失跟踪。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/spread-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-bad"></span>

**错误示例**

`UserSummary.vue` 将 `props.user` 展开为新对象，取得传入响应式数据的快照。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:3"
<script setup lang="ts">
const props = defineProps<{ user: { name: string; role: string } }>();
const copiedUser = { ...props.user };
</script>
```

<span id="vize-croquis-cf-spread-breaks-reactivity-good"></span>

**正确示例**

`toRef(props, "user")` 保留对传入 prop 的引用，不复制其字段。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada", role: "admin" });
</script>

<template>
  <UserSummary :user="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:2,3,5"
<script setup lang="ts">
import { toRef } from "vue";

const props = defineProps<{ user: { name: string; role: string } }>();
const user = toRef(props, "user");
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/suspense-no-fallback`

`<Suspense>` 没有后备内容。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

没有后备内容的 Suspense 是有效的 Vue 语法。这是选定的加载界面规范，并非编译器错误；此公开诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`AsyncCard.vue`

```vue
<script setup lang="ts">
const message = await Promise.resolve('Ready');
</script>

<template>
<p>{{ message }}</p>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-bad"></span>

**错误示例**

Suspense 边界有异步子组件，却没有后备内容，因此此示例的等待状态没有加载内容。

`App.vue`

```vue annotate="remove:7"
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /></Suspense>
</template>

```

<span id="vize-croquis-cf-suspense-no-fallback-good"></span>

**正确示例**

`#fallback` 插槽在异步子组件完成前提供显式的加载段落。

`App.vue`

```vue annotate="add:7"
<script setup lang="ts">
import { Suspense } from 'vue';
import AsyncCard from './AsyncCard.vue';
</script>

<template>
<Suspense><AsyncCard /><template #fallback><p>Loading…</p></template></Suspense>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/template-ref-timing`

模板 ref 在组件挂载之前被读取。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`focus-input.ts`

```ts
export function focusInput(input: HTMLInputElement | null): void { input?.focus(); }

```

<span id="vize-croquis-cf-template-ref-timing-bad"></span>

**错误示例**

setup 在挂载前读取模板 ref，此时值仍为 null，因此可选的 focus 调用不会执行聚焦操作。

`App.vue`

```vue annotate="remove:2,5"
<script setup lang="ts">
import { ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
focusInput(input.value);
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

<span id="vize-croquis-cf-template-ref-timing-good"></span>

**正确示例**

`onMounted` 将读取延迟到 Vue 为模板 ref 赋入输入元素之后，使聚焦辅助函数能作用于该元素。

`App.vue`

```vue annotate="add:2,5"
<script setup lang="ts">
import { onMounted, ref } from 'vue';
import { focusInput } from './focus-input';
const input = ref<HTMLInputElement | null>(null);
onMounted(() => { focusInput(input.value); });
</script>

<template>
<input ref="input" aria-label="Name" />
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/toraw-mutation`

使用 `toRaw` 后修改了原始对象。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { makeProfile, rename } from './profile';
const profile = makeProfile();
</script>

<template>
<p>{{ profile.name }}</p><button @click="rename(profile)">Rename</button>
</template>

```

<span id="vize-croquis-cf-toraw-mutation-bad"></span>

**错误示例**

`rename` 获取原始目标并写入 `raw.name`，绕过本应通知显示名称更新的代理 setter。

`profile.ts`

```ts annotate="remove:1,4,5"
import { reactive, toRaw } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  const raw = toRaw(profile);
  raw.name = 'Grace';
}

```

<span id="vize-croquis-cf-toraw-mutation-good"></span>

**正确示例**

通过传入的响应式代理写入 `profile.name`，保留相同重命名行为，同时通知依赖项。

`profile.ts`

```ts annotate="add:1,4"
import { reactive } from 'vue';
export function makeProfile() { return reactive({ name: 'Ada' }); }
export function rename(profile: { name: string }): void {
  profile.name = 'Grace';
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/uncaught-error`

组件可能抛错，却没有错误边界捕获。

默认严重程度: info  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/uncaught-error": "warn" },
    },
  },
});
```

```sh
vp run lint
```

当前诊断实现扫描 JSON.parse(input) 等模板表达式，不会报告仅存在于 script 块中的 throw 语句。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-uncaught-error-bad"></span>

**错误示例**

子组件模板对格式错误的输入调用 `JSON.parse`，可达父组件没有错误捕获边界。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

<span id="vize-croquis-cf-uncaught-error-good"></span>

**正确示例**

父组件为该子组件注册 `onErrorCaptured`。返回 `false` 会停止传播；生产环境边界还应提供有用的恢复界面。

`App.vue`

```vue annotate="add:2,4"
<script setup lang="ts">
import { onErrorCaptured } from "vue";
import Child from "./Child.vue";
onErrorCaptured(() => false);
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const input = "{";
</script>
<template><button @click="JSON.parse(input)">Parse</button></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/boundary.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/undeclared-emit`

组件发出了未声明的事件。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-emit-bad"></span>

**错误示例**

子组件调用 `emit("save")`，但 `defineEmits` 约定只声明了 `cancel`。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-undeclared-emit-good"></span>

**正确示例**

以空参数元组声明 `save`，使发出的事件符合组件约定。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/undeclared-prop`

父组件传入了子组件未声明的 prop。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-undeclared-prop-bad"></span>

**错误示例**

父组件传入 `typo`，但已解析子组件只声明 `title`。此分析器规范独立于 Vue 通常的属性透传行为。

`App.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" :typo="true" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

<span id="vize-croquis-cf-undeclared-prop-good"></span>

**正确示例**

移除意外的 `typo` 绑定，保留已声明的 `title` prop。

`App.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child title="Hello" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
</script>
<template><p>{{ props.title }}</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/props_validation.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/undefined-slot`

父组件填充了子组件未暴露的插槽。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`Card.vue`

```vue
<script setup lang="ts">
defineSlots<{ header(): unknown }>();
</script>

<template>
<article><header><slot name="header" /></header></article>
</template>

```

<span id="vize-croquis-cf-undefined-slot-bad"></span>

**错误示例**

App 提供 `footer` 插槽，但 Card 仅声明和渲染 `header`，传入的 Notice 内容在此子组件中没有匹配的插槽出口。

`App.vue`

```vue annotate="remove:6"
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #footer>Notice</template></Card>
</template>

```

<span id="vize-croquis-cf-undefined-slot-good"></span>

**正确示例**

App 提供 `header`，与子组件类型化插槽声明和渲染出口一致，因此 Notice 在该处显示。

`App.vue`

```vue annotate="add:6"
<script setup lang="ts">
import Card from './Card.vue';
</script>

<template>
<Card><template #header>Notice</template></Card>
</template>

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unhandled-event`

子组件发出的事件没有父组件处理。

默认严重程度: info  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unhandled-event-bad"></span>

**错误示例**

`Child.vue` 发出 `save`，直属包装组件却没有监听；组件事件不会自动穿过包装组件冒泡。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue annotate="remove:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unhandled-event-good"></span>

**正确示例**

`Wrapper.vue` 为直接子组件附加 `save` 监听器。空回调展示此规则认可的处理形式，并不是完整保存实现。

`App.vue`

```vue
<script setup lang="ts">
import Wrapper from "./Wrapper.vue";
</script>
<template><Wrapper /></template>
```

`Wrapper.vue`

```vue annotate="add:4"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/event_bubbling.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unmatched-inject`

`inject` 指定的键没有任何祖先提供。

默认严重程度: error / warning（具有默认值时）  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unmatched-inject": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-inject-bad"></span>

**错误示例**

`ThemeLabel.vue` 注入 `ThemeKey`，但其可达祖先 `App.vue` 从未提供该键。

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

<span id="vize-croquis-cf-unmatched-inject-good"></span>

**正确示例**

`App.vue` 在渲染注入的后代之前，使用同一导出的 `ThemeKey` 提供响应式 theme。

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";

export interface Theme {
  color: string;
}

export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

`App.vue`

```vue annotate="add:2,4,5,6,7"
<script setup lang="ts">
import { provide, ref } from "vue";
import ThemeLabel from "./ThemeLabel.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unmatched-listener`

父组件监听了子组件不发出的事件。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unmatched-listener-bad"></span>

**错误示例**

父组件监听 `save`，已解析子组件却只声明 `cancel`。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits<{ cancel: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unmatched-listener-good"></span>

**正确示例**

子组件声明并发出 `save`，与父组件监听器名称一致。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child @save="() => {}" /></template>
```

`Child.vue`

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unregistered-component`

模板使用了未注册或导入的组件。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unregistered-component-bad"></span>

**错误示例**

虽然存在 `Child.vue` 文件，父组件既没有导入它，也没有为模板以其他方式注册 `Child`。

`App.vue`

```vue
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unregistered-component-good"></span>

**正确示例**

在父组件 script setup 中导入 `Child`，使模板能解析组件绑定。

`App.vue`

```vue annotate="add:1,2,3"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unresolved-import`

导入无法解析为模块。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unresolved-import-bad"></span>

**错误示例**

父组件导入 `./Missing.vue`，但项目包含的是 `Child.vue`，并非该路径。

`App.vue`

```vue annotate="remove:2"
<script setup lang="ts">
import Child from "./Missing.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

<span id="vize-croquis-cf-unresolved-import-good"></span>

**正确示例**

将导入指向已存在的 `./Child.vue` 文件，保留相同模板绑定。

`App.vue`

```vue annotate="add:2"
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<template><p>Child</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/component_resolution.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unused-attrs`

透传属性传给了未使用它们的多根节点组件。

默认严重程度: info  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-attrs-bad"></span>

**错误示例**

父组件的 `tracking-code` 既未被多根节点子组件作为 prop 使用，也未被转发。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue annotate="remove:1"
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vize-croquis-cf-unused-attrs-good"></span>

**正确示例**

在 `<main>` 上绑定 `$attrs`，为该透传属性提供显式目标。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child tracking-code="notice" /></template>
```

`Child.vue`

```vue annotate="add:1"
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/fallthrough.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unused-emit`

已声明的 emit 从未使用。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

实验性 Rust CrossFileAnalyzer 有此诊断代码的实现。CLI 分析遍不会单独发出该代码；配置其 ID 不会启用该 Rust 分析遍。这些情形描述分析器支持的图与事实，并非 Vite+ 功能承诺。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-unused-emit-bad"></span>

**错误示例**

子组件声明了 `save`，却从未以该名称调用事件发出函数。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
</script>
<template><p>Content</p></template>
```

<span id="vize-croquis-cf-unused-emit-good"></span>

**正确示例**

示例调用 `emit("save")`，使声明事件得到使用。真实交互应在对应操作发生时发出它。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child /></template>
```

`Child.vue`

```vue annotate="add:3"
<script setup lang="ts">
const emit = defineEmits<{ save: [] }>();
emit("save");
</script>
<template><p>Content</p></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/emit.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/unused-provide`

提供的键从未被注入。

默认严重程度: warning  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/unused-provide": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`keys/theme.ts`

```ts
import type { InjectionKey, Ref } from "vue";
export interface Theme { color: string; }
export const ThemeKey: InjectionKey<Ref<Theme>> = Symbol("theme");
```

<span id="vize-croquis-cf-unused-provide-bad"></span>

**错误示例**

`App.vue` 提供 `ThemeKey`，但渲染的 `Dashboard.vue` 子树没有该键的消费者。

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue annotate="remove:2"
<template>
  <h1>Dashboard</h1>
</template>
```

<span id="vize-croquis-cf-unused-provide-good"></span>

**正确示例**

仪表盘现在渲染 `ThemeLabel.vue`，后者注入与祖先完全相同身份的 `ThemeKey`。

`App.vue`

```vue
<script setup lang="ts">
import { provide, ref } from "vue";
import Dashboard from "./Dashboard.vue";
import { ThemeKey, type Theme } from "./keys/theme";

const theme = ref<Theme>({ color: "blue" });
provide(ThemeKey, theme);
</script>

<template>
  <Dashboard />
</template>
```

`Dashboard.vue`

```vue annotate="add:1,2,3,4,6"
<script setup lang="ts">
import ThemeLabel from "./ThemeLabel.vue";
</script>

<template>
  <ThemeLabel />
</template>
```

`ThemeLabel.vue`

```vue annotate="add:1,2,3,4,5,6"
<script setup lang="ts">
import { inject } from "vue";
import { ThemeKey } from "./keys/theme";

const theme = inject(ThemeKey);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/provide_inject/analysis.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/value-extraction-breaks-reactivity`

将响应式值读取到局部变量后丢失后续更新。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/value-extraction-breaks-reactivity": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./UserPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-bad"></span>

**错误示例**

Vue 3.5 响应式解构的 `item` 被一次性读取到 `itemSnapshot`，后续 prop 替换不会更新该快照。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="remove:3"
<script setup lang="ts">
const { item } = defineProps<{ item: { name: string } }>();
const itemSnapshot = item;
</script>
```

<span id="vize-croquis-cf-value-extraction-breaks-reactivity-good"></span>

**正确示例**

在 `computed` 内读取 `item`，让 Vue 的响应式 props 解构转换跟踪每次求值。

`UserPage.vue`

```vue
<script setup lang="ts">
import { reactive } from "vue";
import UserSummary from "./UserSummary.vue";

const user = reactive({ name: "Ada" });
</script>

<template>
  <UserSummary :item="user" />
</template>
```

`UserSummary.vue`

```vue annotate="add:2,3,5"
<script setup lang="ts">
import { computed } from "vue";

const { item } = defineProps<{ item: { name: string } }>();
const itemView = computed(() => item);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/reactivity/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/watch-can-be-computed`

侦听器只将值复制到状态，可以改为计算属性。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

此示例展示公开的纯派生状态偏好。侦听器仍适用于外部副作用或独立可写状态；目前没有实现发出此诊断约定。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import { useDouble } from './use-double';
const { count, doubled } = useDouble();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ doubled }}</p>
</template>

```

<span id="vize-croquis-cf-watch-can-be-computed-bad"></span>

**错误示例**

侦听器没有外部副作用，只将第二个可写 ref 同步为 `count` 的两倍。示例没有对该派生值的独立写入。

`use-double.ts`

```ts annotate="remove:1,4,5"
import { ref, watch } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = ref(0);
  watch(count, next => { doubled.value = next * 2; }, { immediate: true });
  return { count, doubled };
}

```

<span id="vize-croquis-cf-watch-can-be-computed-good"></span>

**正确示例**

计算 getter 直接表达相同派生关系，移除手动同步及额外可写状态。

`use-double.ts`

```ts annotate="add:1,4"
import { computed, ref } from 'vue';
export function useDouble() {
  const count = ref(0);
  const doubled = computed(() => count.value * 2);
  return { count, doubled };
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/watcheffect-async`

`watchEffect` 启动异步任务，却无法清理上一轮执行。

默认严重程度: error  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "croquis/cf/watcheffect-async": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./SearchPage.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

`api.ts`

```ts
export interface Result { items: string[]; }
export async function load(query: string, options?: { signal?: AbortSignal }): Promise<Result> {
  const response = await fetch(`/search?q=${encodeURIComponent(query)}`, options);
  return response.json();
}
```

<span id="vize-croquis-cf-watcheffect-async-bad"></span>

**错误示例**

异步 `watchEffect` 将隐式依赖收集与等待请求混合，且没有失效保护。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="remove:3,8,9,10"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watchEffect } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watchEffect(async () => {
  result.value = await load(props.query);
});
</script>
```

<span id="vize-croquis-cf-watcheffect-async-good"></span>

**正确示例**

显式的 `watch(() => props.query, ...)` 声明来源、注册请求清理，并在失效后拒绝过期响应。

`SearchPage.vue`

```vue
<script setup lang="ts">
import { ref } from "vue";
import SearchResults from "./SearchResults.vue";

const query = ref("");
</script>

<template>
  <SearchResults :query="query" />
</template>
```

`SearchResults.vue`

```vue annotate="add:3,8,9,10,11,12,13,14,15,16,17,18,19,20,21,22"
<script setup lang="ts">
import { load, type Result } from "./api";
import { ref, watch } from "vue";

const props = defineProps<{ query: string }>();
const result = ref<Result | null>(null);

watch(
  () => props.query,
  async (value, _oldValue, onCleanup) => {
    const controller = new AbortController();
    let active = true;

    onCleanup(() => {
      active = false;
      controller.abort();
    });

    const next = await load(value, { signal: controller.signal });
    if (active) result.value = next;
  },
);
</script>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[诊断实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_croquis_cf/src/rules/race_conditions/diagnostics.rs)

[跨文件规则索引](cross-file.md)

### `vize:croquis/cf/watcher-outside-setup`

`watch` 或 `watchEffect` 在 `setup` 外被调用。

默认严重程度: 不产生诊断  
适用范围: 已分析的组件图，以及下方说明的受支持事实  
自动修复: 无；请检查相关文件并应用修复  
选项: 无针对单个诊断代码的选项；受支持的 CLI 诊断可覆盖严重程度

这是目前没有实现的公开诊断约定。下方错误与正确示例解释风险及修复；目前没有任何标志能使此诊断代码触发。

所有者保留并调用停止句柄，或有意赋予应用级生命周期时，模块作用域侦听器是有效的。此示例要求组件拥有生命周期；该诊断约定目前没有实现。

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from 'vue';
import App from './App.vue';
createApp(App).mount('#app');

```

`index.html`

```html
<!doctype html>
<html lang="en"><head><meta charset="UTF-8"><title>Contract scenario</title></head>
<body><div id="app"></div><script type="module" src="/main.ts"></script></body></html>

```

`App.vue`

```vue
<script setup lang="ts">
import Observer from './Observer.vue';
</script>

<template>
<Observer /><Observer />
</template>

```

`Observer.vue`

```vue
<script setup lang="ts">
import { useObserver } from './use-observer';
const { count, observed } = useObserver();
</script>

<template>
<button @click="count++">{{ count }}</button><p>{{ observed }}</p>
</template>

```

<span id="vize-croquis-cf-watcher-outside-setup-bad"></span>

**错误示例**

侦听器在模块加载时创建，位于两个 Observer 的 setup 之外，两实例共享其 refs。某个 Observer 卸载时不会自动停止它。

`use-observer.ts`

```ts annotate="remove:2,3,4,5"
import { ref, watch } from 'vue';
const count = ref(0);
const observed = ref(0);
watch(count, next => { observed.value = next; });
export function useObserver() { return { count, observed }; }

```

<span id="vize-croquis-cf-watcher-outside-setup-good"></span>

**正确示例**

每个同步 setup 调用在 `useObserver` 中创建自身 refs 和侦听器。Vue 将该侦听器与调用组件的生命周期关联。

`use-observer.ts`

```ts annotate="add:2,3,4,5,6,7"
import { ref, watch } from 'vue';
export function useObserver() {
  const count = ref(0);
  const observed = ref(0);
  watch(count, next => { observed.value = next; });
  return { count, observed };
}

```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[公开说明](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize/src/commands/explain/snapshots/en.txt)

[跨文件规则索引](cross-file.md)

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

### `vue/cross-file-attrs-fallthrough`

父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。

默认严重程度: warning  
适用范围: 可达的项目声明及导入组件  
选项: crossFile；规则严重程度（off/warn/error）  
自动修复: 无

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: { preset: "incremental", crossFile: true,
      rules: { "vue/cross-file-attrs-fallthrough": "warn" },
    },
  },
});
```

```sh
vp run lint
```

**共享项目文件**

错误和正确示例都使用这些不变的文件。在项目中安装导入的包：Vue，以及示例中出现的 vue-router 或 Pinia。遵循版本特定的支持说明。入口根文件显式展示组件关系。

`main.ts`

```ts
import { createApp } from "vue";
import Root from "./App.vue";
createApp(Root).mount("#app");
```

`index.html`

```html
<div id="app"></div>
<script type="module" src="/main.ts"></script>
```

<span id="vue-cross-file-attrs-fallthrough-bad"></span>

**错误示例**

父组件将 `class="notice"` 传给已解析的片段子组件；该子组件没有自动属性目标，也从不读取 `$attrs`。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="remove:1"
<template><main>Content</main><aside>Help</aside></template>
```

<span id="vue-cross-file-attrs-fallthrough-good"></span>

**正确示例**

子组件将 `$attrs` 绑定在 `<main>` 上，选择它作为目标；同级 `<aside>` 保持独立。

`App.vue`

```vue
<script setup lang="ts">
import Child from "./Child.vue";
</script>
<template><Child class="notice" /></template>
```

`Child.vue`

```vue annotate="add:1"
<template><main v-bind="$attrs">Content</main><aside>Help</aside></template>
```

正确示例的文件展示上述修改；完整项目仍可能适用其他诊断。

[跨文件规则索引](cross-file.md)

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
