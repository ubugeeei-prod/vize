export const chineseRules1: Record<string, readonly [string, string, string]> = {
  "css/prefer-logical-properties": [
    "建议使用 CSS 逻辑属性，以更好地支持国际化",
    "`margin-left` 无论书写方向如何，都将外边距固定在物理侧。",
    "`margin-inline-start` 改为跟随行内方向的起始侧。",
  ],
  "css/prefer-nested-selectors": [
    "建议为后代选择器使用 CSS 嵌套",
    "`.card .title` 后代选择器在平铺规则中重复了父选择器。",
    "`.title` 规则嵌套在 `.card` 内，将父子样式关系放在一起。",
  ],
  "css/prefer-slotted": [
    "建议使用 ::v-slotted() 为插槽内容设置样式",
    "scoped 样式表选择的是 `slot` 出口，而不是通过插槽传入的元素。",
    "`:slotted(.label)` 通过作用域插槽选择器选中传入的 label 元素。",
  ],
  "css/require-font-display": [
    "要求 @font-face 规则具有 font-display",
    "font-face 声明定义了字体来源，却省略了 font-display 策略。",
    "`font-display: swap` 显式选择先显示后备字体、再切换到目标字体的策略。",
  ],
  "ecosystem/nuxt-prefer-nuxt-link": [
    "应用内部链接优先使用 NuxtLink",
    "Nuxt 应用中的内部设置目标使用了普通锚点。",
    "NuxtLink 通过 Nuxt 路由处理相同的内部目标。",
  ],
  "ecosystem/pinia-prefer-store-to-refs": [
    "解构 Pinia store 时优先使用 storeToRefs()",
    "直接从 store 解构 `name`，使该值脱离了响应式 store 访问。",
    "store 保持完整，storeToRefs 为 name 创建响应式引用。",
  ],
  "ecosystem/router-link-require-to": [
    "要求 RouterLink 和 NuxtLink 组件具有 `to` 目标",
    "嵌套 RouterLink 没有 `to` 目标，不能依赖根节点的属性透传。",
    '`to="/settings"` 显式提供嵌套链接目标。',
  ],
  "ecosystem/void-link-require-href": [
    "要求 Void Vue Link 组件具有 `href`",
    "从 @void/vue 导入的 Link 省略了 href 目标。",
    "同一导入的 Link 通过 href 接收设置目标。",
  ],
  "ecosystem/void-link-valid-method": [
    "验证 Void Vue Link 的静态 method prop",
    "DELETE 操作请求了预取，但预取适用于导航请求。",
    "移除 prefetch，保留 DELETE 操作，不再预取该非 GET 请求。",
  ],
  "ecosystem/vue-i18n-no-missing-key": [
    "报告本地 SFC 消息中不存在的静态 vue-i18n 键",
    "模板请求 auth.missing，但本地英文消息只声明了 auth.login。",
    "模板请求本地消息中已存在的 auth.login 键。",
  ],
  "ecosystem/vue-router-prefer-named-link": [
    "RouterLink 优先使用具名路由对象，而不是静态路径字符串",
    "RouterLink 的目标是路径字面量，而不是具名路由。",
    "绑定的路由对象通过 settings 路由名称标识目标。",
  ],
  "ecosystem/vue-router-prefer-named-push": [
    "Vue Router 编程式导航优先使用具名路由对象",
    "router.push 接收与当前 URL 写法绑定的路径字符串。",
    "router.push 接收具有稳定 settings 名称的路由对象。",
  ],
  "ecosystem/vue-test-utils-no-html-snapshot": [
    "避免在 Vue Test Utils 测试中对 wrapper.html() 生成快照",
    "断言对整个 wrapper HTML 生成快照，而不是检查预期行为。",
    "断言检查渲染文本是否包含 Saved。",
  ],
  "html/deprecated-attr": [
    "禁止已弃用的 HTML 属性",
    "段落使用了已弃用的呈现属性 `align`。",
    "类名和 `text-align: center` 声明通过 CSS 表达对齐方式。",
  ],
  "html/deprecated-element": [
    "禁止已弃用的 HTML 元素",
    "`center` 使用了已弃用的 HTML 呈现元素。",
    "section 和样式类替代弃用元素，同时保留内容。",
  ],
  "html/id-duplication": [
    "禁止重复的元素 ID",
    '输入框和帮助段落都声明 `id="email"`，使标签目标不明确。',
    "输入框保留 `email`；帮助段落使用 `email-help`，aria-describedby 引用该独立 ID。",
  ],
  "html/no-consecutive-br": [
    "禁止连续的 &lt;br&gt; 元素",
    "两个连续换行元素在同一段落内为内容块制造间距。",
    "使用独立段落表达两个内容块，无需重复换行元素。",
  ],
  "html/no-dupe-style-properties": [
    "禁止内联 style 属性中重复的属性声明",
    "每个静态 style 都重复了一个属性；`margin` 和 `MARGIN` 也算作同一属性。",
    "静态 style 使用不同的 color 和 background 属性。动态样式绑定不在此静态属性检查范围内。",
  ],
  "html/no-duplicate-class": [
    "禁止静态 class 属性中重复的类名",
    "静态类名列表重复了 `btn`。",
    "类名列表保留一个 `btn` 及不同的 `primary`。",
  ],
  "html/no-duplicate-dt": [
    "禁止 &lt;dl&gt; 中重复的 &lt;dt&gt; 名称",
    "同一定义列表为两项描述重复了 `API` 术语。",
    "一个 API 术语后跟两项描述，避免重复术语。",
  ],
  "html/no-empty-palpable-content": [
    "禁止预期具有可见内容的空元素",
    "段落、列表项和表格单元格都没有可感知的内容。",
    "文本填充段落，插值提供列表项内容，aria-label 则显式命名原本为空的单元格。",
  ],
  "html/require-datetime": [
    "要求 &lt;time&gt; 元素具有 datetime 属性",
    "time 元素包含人类可读的日期，却没有机器可读的 datetime 值。",
    '`datetime="2026-05-13"` 提供对应的机器可读日期。',
  ],
  "musea/no-empty-variant": [
    "禁止空的 &lt;variant&gt; 块",
    "名为 primary 的 variant 为空，没有提供预览内容。",
    "variant 渲染带有 Save 内容的 primary Button。",
  ],
  "musea/prefer-design-tokens": [
    "优先使用设计令牌 CSS 变量，而不是硬编码的原始值",
    "art 示例使用蓝色字面量，而不是配置的 primary 设计令牌。",
    "样式引用此示例配置的令牌 --color-primary。",
  ],
  "musea/require-component": [
    "要求 &lt;art&gt; 块具有 component 属性",
    "art 块提供了标题，却未标识要预览的组件。",
    "defineArt 将 ./Button.vue 指定为 art 块的组件。",
  ],
  "musea/require-title": [
    "要求 &lt;art&gt; 块具有 title 属性",
    "art 块标识了 Button.vue，却没有提供标题。",
    "defineArt 选项为 art 块提供 Button 标题。",
  ],
  "musea/unique-variant-names": [
    "要求 variant 名称唯一",
    "同一 art 块中的两个 variant 都使用 primary 名称。",
    "两个 variant 分别使用不同的 primary 和 secondary 名称。",
  ],
  "musea/valid-variant": [
    "要求 &lt;variant&gt; 块具有 name 属性",
    "variant 省略了标识预览所需的名称。",
    "primary 名称标识了该 variant。",
  ],
  "nuxt/no-nuxt-config-test-key": [
    "禁止在 Nuxt 配置中设置 `test` 键",
    "导出的 Nuxt 配置将标识符键 `test` 设为布尔值 `true`，这是此规则拒绝的旧配置形式。",
    "空配置移除了该布尔 `test` 属性。此示例并不禁止测试配置对象。",
  ],
  "nuxt/no-page-meta-runtime-values": [
    "禁止在 `definePageMeta` 的立即求值层使用运行时上下文值；该层在构建时被提取到独立代码块，并在组件 setup 之前运行",
    "构造 `definePageMeta` 对象时立即执行 `useRoute()`，但宏会将元数据提升到 setup 运行时上下文之外。",
    "`validate` 接收回调，因此 `useRoute().params.id` 访问延迟到回调运行时。规则区分延迟执行的函数体和立即求值的元数据值。",
  ],
  "nuxt/nuxt-config-keys-order": [
    "优先使用推荐的 Nuxt 配置属性顺序",
    "配置将 `ssr` 放在 `modules` 之前，与规则推荐的 Nuxt 配置键顺序相反。",
    "将 `modules` 放在 `ssr` 之前，在保留两个值的同时满足规定顺序；修复改变布局，不改变选项含义。",
  ],
  "nuxt/prefer-import-meta": [
    "优先使用 `import.meta.*`，而不是 `process.*`",
    "`process.client` 使用旧 Nuxt 环境标志，规则要求将其迁移到 `import.meta`。",
    "`import.meta.client` 通过替代环境标志显式保留仅浏览器执行的分支。",
  ],
  "petite-vue/no-unsupported-directive": [
    "禁止 petite-vue 不支持的指令",
    "`v-memo`、`v-slot:header` 和自定义 `v-my-directive` 不在 petite-vue 支持的指令列表中。petite-vue 脚本将此 HTML 标识为相应方言。",
    "替换后使用受支持的 `v-scope`、`v-effect`、`v-if`、`v-bind` 和 `v-on` 语法，不再依赖不支持的指令。",
  ],
  "petite-vue/valid-v-effect": [
    "要求 v-effect 具有非空表达式",
    "每个 `v-effect` 都没有可执行表达式：值缺失、为空，或只包含空白。",
    "两个 `v-effect` 值都包含表达式：一个更新 `el.textContent`，另一个递增 `count`。规则检查表达式非空，不检查副作用的业务逻辑。",
  ],
  "petite-vue/valid-v-scope": [
    "要求 v-scope 绑定对象字面量",
    "四个非空 `v-scope` 值分别是标识符、调用、算术表达式和数字；都不能解析为对象字面量。",
    "无值的 `v-scope` 使用根作用域。其他值是对象字面量，包括规则接受的加括号对象。",
  ],
  "script/component-options-name-casing": [
    "要求组件 `name` 选项使用 PascalCase",
    "组件选项 `name: 'my-component'` 使用 kebab-case，但此规则要求名称字面量采用 PascalCase。",
    "`MyComponent` 以大写字母开头，且仅包含字母和数字，满足名称检查。",
  ],
};
