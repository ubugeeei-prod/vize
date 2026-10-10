export const chineseRules2: Record<string, readonly [string, string, string]> = {
  "script/custom-event-name-casing": [
    "要求发出的自定义事件名称使用 camelCase",
    "发出的字符串 `my-event` 含有连字符，违反默认的 camelCase 事件命名策略。",
    "声明和调用都使用 `myEvent`，既保持事件名称与发出操作一致，也满足默认大小写策略。配置为 kebab-case 时要求不同。",
  ],
  "script/define-emits-declaration": [
    "要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式",
    '`defineEmits(["change"])` 使用运行时数组声明；此风格规则偏好类型声明。',
    '`defineEmits<{ change: [id: number] }>()` 将事件声明移到类型参数中，并显式描述 `emit("change", 1)` 使用的数字载荷。',
  ],
  "script/define-macros-order": [
    "要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序",
    "`defineProps` 位于 `defineModel` 之前，但标准宏顺序中 `defineModel` 更靠前。",
    "声明严格按照 `defineOptions`、`defineModel`、`defineProps`、`defineEmits`、`defineSlots` 排列，再写无关的运行时语句。",
  ],
  "script/define-props-declaration": [
    "要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式",
    "`defineProps({ title: String })` 提供运行时对象，与此规则偏好的类型 props 声明冲突。",
    "`defineProps<{ title: string }>()` 在类型参数中声明 `title`，保留 `props.title` 访问，无需运行时声明参数。",
  ],
  "script/define-props-destructuring": [
    "要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致",
    "`defineProps` 被赋给单一的 `props` 绑定，而非解构，不符合默认的解构偏好。",
    "对象模式直接绑定 `foo` 和 `bar`，并为可选的 `bar` 提供默认值。这依赖 Vue 3.5+ 的响应式 props 解构；可配置的 `never` 模式偏好相反形式。",
  ],
  "script/no-arrow-functions-in-watch": [
    "禁止将箭头函数用作 Options API watch 处理器",
    "Options API 侦听器 `value` 和嵌套的 `other.handler` 都是箭头函数。箭头函数捕获外层 `this`，不会接收组件实例。",
    "两个处理器改为普通方法，使 Vue 能将 `this` 绑定到组件。`deep: true` 侦听选项仍与对象形式兼容。",
  ],
  "script/no-async-in-computed": [
    "禁止计算属性中的异步函数",
    "`computed` getter 使用 `async`，因此 fetch 产生 Promise，而不是同步派生的计算值。",
    "异步 fetch 移到 `watch`，将结果存入 `data.value`。清理逻辑中止旧请求，并阻止失效回调写入过期结果；不再存在异步计算 getter。",
  ],
  "script/no-boolean-default": [
    "禁止为 Boolean prop 设置默认值",
    "`disabled` 和 `checked` 都为唯一构造器为 `Boolean` 的 prop 声明了 `default`；规则连显式的 `false` 默认值也会拒绝。",
    "仅 Boolean 的 props 省略 `default`，使用 Vue 隐式的 false 值。`[Boolean, String]` 联合类型和 Number prop 说明此检查仅限唯一的 `Boolean` 构造器。",
  ],
  "script/no-deep-destructure-in-props": [
    "禁止在 defineProps 中进行深层嵌套解构",
    "绑定模式进入 `user` 解构 `name`，超过默认的浅层 props 解构深度。",
    "props 对象保持完整，由计算 getter 读取 `props.user.name`。嵌套访问保持显式，不使用深层嵌套绑定模式。",
  ],
  "script/no-deprecated-data-object-declaration": [
    "禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）",
    "Options API 的 `data` 选项是对象字面量，这是 Vue 3 不再接受的 Vue 2 形式。",
    "`data()` 返回新的 `{ count: 0 }` 对象，提供 Vue 3 所需的函数式 data 声明。",
  ],
  "script/no-deprecated-destroyed-lifecycle": [
    "禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子",
    "定时器清理使用了已移除的 Vue 2 生命周期选项 `beforeDestroy`。",
    "将钩子重命名为 `beforeUnmount`，在 Vue 3 生命周期名称下保留清理函数体。",
  ],
  "script/no-deprecated-dollar-listeners-api": [
    "禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）",
    "成员读取和独立参数引用都使用 `$listeners`，Vue 3 在将监听器合并到属性后移除了它。",
    "读取改为 `this.$attrs` 和 setup 上下文的 `ctx.attrs`，替代已移除的监听器接口；示例中的接收对象必须存在于周围组件上下文。",
  ],
  "script/no-deprecated-dollar-scopedslots-api": [
    "禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）",
    "`this.$scopedSlots`、`ctx.$scopedSlots` 和独立的 `$scopedSlots` 引用使用了 Vue 3 已移除的 Vue 2 作用域插槽 API。",
    "将 `$scopedSlots` 替换为 `$slots`，使用统一插槽接口。示例移除弃用写法，并未为接收对象建立 setup 上下文。",
  ],
  "script/no-deprecated-events-api": [
    "禁止已移除的 Vue 2 事件 API（$on / $off / $once）",
    "`$on`、`$once` 和 `$off` 调用使用了 Vue 3 已移除的实例事件总线方法。",
    "`$emit` 仍然有效，事件总线订阅则移到外部发射器的 `on` 方法。修复区分面向父组件的事件发出与外部事件总线。",
  ],
  "script/no-deprecated-props-default-this": [
    "禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）",
    "prop 默认值和校验函数读取 `this`，但在 Vue 3 中这些函数不能依赖组件实例。",
    "默认值从参数读取 `props.baseSize`，校验函数检查 `value` 参数。两者都不再依赖不可用的实例接收对象。",
  ],
  "script/no-dupe-keys": [
    "禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键",
    "props 和 data 都声明 `foo`，computed 和 methods 都声明 `bar`。这些声明竞争相同的组件实例键。",
    "prop、data 和 computed 声明使用不同名称（`foo`、`bar` 和 `baz`），消除两处跨选项冲突。",
  ],
  "script/no-duplicate-attr-inheritance": [
    "标记重复应用透传属性的组件",
    "显式 `inheritAttrs: true` 重复了 Vue 的默认值。即使没有展示根节点 `$attrs` 展开，规则也会报告此多余字面量。",
    "`inheritAttrs: false` 表达实际的退出选择，空选项对象则隐式保留默认继承。两者都不重复多余的 `true`。",
  ],
  "script/no-export-in-script-setup": [
    "禁止 &lt;script setup&gt; 内的 export 语句",
    "`export const count` 试图从 `<script setup>` 暴露模块导出，但这里禁止运行时导出。",
    "移除 `export`，使 `count` 保持为 setup 绑定，而非模块导出。",
  ],
  "script/no-get-current-instance": [
    "禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）",
    "带 Vapor 标记的 setup 导入并调用 `getCurrentInstance`，依赖了此规则针对 Vapor 组件禁止的实例 API。",
    '`inject("app-config")` 获取显式提供的配置，无需导入或调用 `getCurrentInstance`。',
  ],
  "script/no-import-compiler-macros": [
    "禁止导入自动可用的 Vue 编译器宏",
    "`vue` 导入包含 `defineProps` 和 `defineEmits`，但这些编译器宏已在 `<script setup>` 中直接可用。",
    "移除宏导入，保留两个带类型的宏调用；两种声明都无需运行时导入。",
  ],
  "script/no-internal-imports": [
    "禁止从 Vue 内部模块导入",
    "两个导入都指向内部 `dist` 文件，而非 Vue 公共包入口，使组件与构建文件路径耦合。",
    "从 `vue` 导入所需辅助函数，移除对内部发行文件位置的依赖。",
  ],
  "script/no-multiple-slot-args": [
    "禁止向作用域插槽函数调用传入多个参数",
    "插槽调用传入多个位置参数，或展开未知参数列表。Vue 插槽接收一个 props 对象，而非位置参数列表。",
    "`{ foo, bar }` 将数据合并为一个参数；`slotProps` 和无参数调用也符合支持的插槽调用形式。",
  ],
  "script/no-next-tick": [
    "禁止面向 Vapor 的组件使用 nextTick()",
    "面向 Vapor 的组件导入并等待 `nextTick`，引入此迁移规则拒绝的 DOM 刷新调度依赖。",
    "通过 `useTemplateRef` 获取输入框，并在 `onMounted` 时聚焦。显式挂载边界替代示例中的 `nextTick` 依赖。",
  ],
  "script/no-options-api": [
    "禁止 Vapor 模式中的 Options API 模式",
    "默认导出对象声明了 Options API 的 `data()`，这是此规则禁止的组件选项形式。",
    "组件状态改为 Vapor `<script setup>` 中的 Composition API `ref`，移除 Options API 对象及其 `data` 选项。",
  ],
  "script/no-potential-component-option-typo": [
    "标记 Options API 组件选项名称中的疑似拼写错误",
    "选项被写为 `method`，与受识别的 `methods` 选项只差一次编辑；Vue 不会将它视为预期的方法声明。",
    "将键改为 `methods`，使 `save()` 位于受识别的组件选项下。",
  ],
  "script/no-reactive-destructure": [
    "禁止导致响应性丢失的响应式对象解构",
    "`const { count, name } = state` 从 `reactive` 对象复制原始类型属性，失去与后续属性变化的联系。",
    "解构 `toRefs(state)` 为 `count` 和 `name` 创建 refs，使每个绑定继续关联原始响应式属性。",
  ],
  "script/no-ref-as-operand": [
    "要求将 ref 绑定变量用作操作数时通过 `.value` 访问",
    "`count + 1` 将 ref 对象本身用作算术操作数，而不是其包裹的数字。",
    "`count.value + 1` 先读取包裹的数字，再加一；脚本算术需要此显式 ref 访问。",
  ],
  "script/no-required-prop-with-default": [
    "禁止 prop 同时具有 required: true 和默认值",
    '`title` 既是必需项，又具有后备值 `"Untitled"`，混合了必需输入约定和面向缺失输入的默认值。',
    '移除 `required: true`，使 `title` 可选，并保留 `"Untitled"` 作为一致的后备值。',
  ],
  "script/no-reserved-identifiers": [
    "禁止使用 Vue 编译器保留标识符",
    "绑定 `__props`、`__emit` 和 `__sfc__` 使用了为 Vue 编译器生成代码保留的标识符。",
    "普通名称 `props`、`emit` 和 `componentData` 避开这些生成标识符，同时保留 props 和 emits 声明。",
  ],
  "script/no-reserved-keys": [
    "禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键",
    "返回的 data 键 `$el` 与 Vue 内置组件实例属性冲突，也使用了保留的 `$` 前缀。",
    "将应用数据重命名为 `elementLabel`，避开内置实例接口和保留前缀。",
  ],
  "script/no-reserved-props": [
    "禁止在组件 props 声明中使用保留名称",
    "对象形式的 `ref` 和 `$foo`，以及数组形式的 `key`，都是保留 prop 名称。`ref` 和 `key` 是框架控制项，带 `$` 前缀的名称也会被拒绝。",
    "普通 prop 名称 `name` 和 `refValue` 在拼写及前缀上都避开了保留名称。",
  ],
  "script/no-restricted-globals": [
    "禁止直接引用必须通过类型化包装器访问的运行时环境全局变量",
    "示例直接读取默认受限全局变量 `process`、`localStorage` 和 `sessionStorage`，绕过项目显式的配置和存储辅助函数。",
    "`useFeatureFlag`、`authStorage.read` 和 `viewStorage.write` 移除了这些直接引用。剩余的 `window.scrollY` 不是规则的默认限制；SSR 安全是独立问题。",
  ],
  "script/no-restricted-members": [
    "禁止项目配置的 object.property 成员访问",
    '在 `ruleOptions` 中配置 `{ object: "window", property: "localStorage" }` 后，`window.localStorage` 访问了禁止的对象与成员组合。规则没有默认禁止成员。',
    '`authStorage.read("token")` 将读取交给应用存储辅助函数，不再访问配置禁止的 `window.localStorage` 成员。',
  ],
  "script/no-side-effects-in-computed-properties": [
    "禁止 Options API 计算 getter 中的副作用",
    "`doubled` 赋值给 `this.count`，`reversed` 通过 `reverse()` 修改 `this.items`。两个 getter 都修改了本应只用于派生的状态。",
    "`doubled` 返回乘法结果，不进行赋值。`reversed` 先复制数组再反转，getter 不会改变原组件状态。",
  ],
  "script/no-top-level-ref-in-script": [
    "禁止顶层 ref/reactive，以防止跨请求状态污染",
    "普通 `<script>` 在模块作用域初始化 `count` 和 `user`。SSR 期间，这些状态对象可能跨组件实例和请求共享。",
    "setup ref 按组件实例初始化；普通脚本只保留常量、生成状态的函数，以及 `setup()` 内创建的 ref。都不会在普通模块作用域创建响应式状态。",
  ],
  "script/no-unstable-nested-components": [
    "禁止在 setup 或 render 函数内定义组件",
    "`defineComponent` 在父组件的 `setup()` 内运行，每次执行 setup 都创建新的 `Child` 组件定义。",
    "`Child` 定义移到模块作用域，`setup()` 返回已存在的定义。",
  ],
};
