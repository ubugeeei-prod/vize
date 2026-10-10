export const chineseRules3: Record<string, readonly [string, string, string]> = {
  "script/no-unused-emit-declarations": [
    "标记已声明却从未发出的事件",
    "`defineEmits` 声明了 `change` 和 `unused`，但捕获的 `emit` 函数只发出字面量事件 `change`。",
    "移除 `unused`，使声明的事件列表与观察到的发出操作一致。示例使用捕获且未逸出的 emit 绑定，因此可以得出此局部使用结论。",
  ],
  "script/no-use-computed-property-like-method": [
    "禁止像方法一样调用 Options API 计算属性",
    "`this.total()` 调用了计算 getter 暴露的值；getter 返回不可调用的 `3`。",
    "`this.total` 不加调用括号，直接读取计算值，使 `log` 打印派生数字。",
  ],
  "script/no-with-defaults": [
    "不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）",
    "`withDefaults` 包装带类型的 props 声明，仅为 `count` 和 `name` 提供默认值，没有使用此处偏好的 Vue 3.5+ 解构默认值风格。",
    '解构模式将 `count = 0` 和 `name = "Ada"` 放在对应绑定旁，并移除 `withDefaults` 包装。',
  ],
  "script/prefer-computed": [
    "响应式派生状态优先使用 computed()",
    "侦听器仅将 `count` 的派生值复制到第二个 ref `doubled`，通过手动同步维护派生状态。",
    "`computed(() => count.value * 2)` 直接表达派生关系，移除额外可写 ref 及其同步侦听器。",
  ],
  "script/prefer-define-options": [
    "优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;",
    "普通脚本唯一有实质作用的语句导出只含 `name` 和 `inheritAttrs` 的对象；这些选项可通过 `defineOptions` 表达。",
    "示例中的 `data()` 方法让脚本包含实际 Options API 逻辑，因此不在此规则保守的仅选项建议范围内。此正确示例展示允许的例外；直接迁移则应在 `<script setup>` 中使用 `defineOptions({ name: 'MyComponent', inheritAttrs: false })`。",
  ],
  "script/prefer-import-from-vue": [
    "优先从 'vue' 导入，而不是内部包",
    "`ref` 和 `h` 从内部 `@vue/runtime-core` 和 `@vue/runtime-dom` 包导入，而不是公共 `vue` 包。",
    "两个辅助函数一起从 `vue` 导入，使用公共包入口，不使用任何内部包。",
  ],
  "script/prefer-ref-over-reactive": [
    "建议使用 ref() 而不是 reactive() 管理状态",
    "状态通过 `reactive` 创建，不符合此主张型规则对 refs 的偏好。示例展示风格偏好，并非响应式对象本身无效。",
    "示例通过 `ref` 创建标量和对象状态；相关字段也可拆为独立 refs，满足偏好的状态构造形式。",
  ],
  "script/prefer-use-attrs": [
    "建议使用 useAttrs() 而不是 context.attrs",
    "`setup` 通过解构上下文参数获取 `attrs`，规则要求改用 Composition API 辅助函数。",
    "`useAttrs()` 在 setup 内提供 `attrs`，保留 `attrs.class` 读取，不依赖第二个 setup 参数。",
  ],
  "script/prefer-use-id": [
    "建议使用 useId() 生成唯一 ID（Vue 3.5+）",
    "`id` 包含 `Math.random()`，生成的输入框与标签标识符可能在服务端和客户端渲染时不同。以 ID 命名的绑定是规则可识别的生成上下文。",
    "Vue 3.5+ 的 `useId()` 生成标识符，`:for` 和 `:id` 继续读取同一绑定，不再分别生成随机值。",
  ],
  "script/prefer-use-slots": [
    "建议使用 useSlots() 而不是 context.slots",
    "`setup` 从上下文参数解构 `slots`，这是此规则建议替换的访问形式。",
    "`useSlots()` 在 setup 内获取插槽，保留渲染函数及可选默认插槽调用，不使用上下文参数。",
  ],
  "script/prefer-use-template-ref": [
    "模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+）",
    '可为空的 `input` ref 与模板字面量 `ref="input"` 配对，表明它是元素引用，而非普通可空数据。',
    "Vue 3.5+ 的 `useTemplateRef<HTMLInputElement>('input')` 显式表达模板引用。未配对的 `error = ref(null)` 仍是普通数据，有意不在此规则范围内。",
  ],
  "script/require-default-prop": [
    "要求每个可选的非 Boolean prop 具有默认值",
    "`name` 和 `age` 是没有默认值的可选非 Boolean 运行时 props，省略输入时的值未指定。",
    "`name` 获得 `default: ''`。`enabled` 使用 Boolean 隐式的 false 默认值，必需的 `id` 无需后备值，展示了两种豁免。",
  ],
  "script/require-explicit-emits": [
    "要求发出的事件在 defineEmits 或 emits 选项中声明",
    "捕获的 emit 函数发出 `save`，但 `defineEmits([])` 未声明该事件。",
    '在声明中添加 `"save"`，使发出的字面量事件成为组件显式事件约定的一部分。',
  ],
  "script/require-explicit-slots": [
    "要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型",
    "带类型的 `defineProps<{ id: number }>()` 确立了 TypeScript 语法，但 setup 使用 `useSlots()` 而没有 `defineSlots` 声明，因此规则发现缺少显式插槽约定。",
    "`defineSlots` 声明 props 含 `msg: string` 的 `default` 插槽；`useSlots()` 现在配有显式的类型化插槽约定。",
  ],
  "script/require-function-return-type": [
    "要求函数具有返回类型注解",
    "`add` 和 `greet` 都注解了参数，却省略返回类型注解；推断返回类型不满足此显式注解策略。",
    "`add` 声明 `: number`，`greet` 声明 `: string`，明确返回约定而不改变函数体。",
  ],
  "script/require-prop-type-constructor": [
    "要求 prop 的 `type` 值是构造器，而不是字符串字面量",
    'prop 声明将字符串 `"String"` 和 `"Number"` 用作运行时类型，包括构造器数组内的值。这些字符串不是构造函数。',
    "声明使用实际的 `String` 和 `Number` 标识符，包括联合数组 `[String, Number]`。",
  ],
  "script/require-prop-types": [
    "要求每个 prop 声明类型",
    "数组项只声明名称 `status`；`null` 值和空描述对象也没有声明运行时 prop 类型。",
    "`status: String` 提供简写构造器，`other` 在描述对象内提供 `type: Number`，两个 props 现在都有类型声明。",
  ],
  "script/require-symbol-provide": [
    "建议使用 Symbol 作为 provide/inject 的注入键",
    "`provide` 和 `inject` 使用 `'user'`、`'theme'` 等字符串字面量键，可能与同名的其他提供者冲突。",
    "共享的 `UserKey` 通过 `Symbol` 创建，并注解为 `InjectionKey<User>`；两个调用都传入该键，而非字符串字面量。",
  ],
  "script/require-typed-object-prop": [
    "要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型",
    "单独的 `Object` 和 `Array` 构造器只描述宽泛的运行时类别，因此 `user` 和 `items` 元素结构都没有显式静态类型。",
    "`PropType<User>` 和 `PropType<User[]>` 添加对象和元素类型，同时保留原运行时构造器。",
  ],
  "script/require-typed-ref": [
    "要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数",
    "导入的 `ref` 调用既无类型参数，也无有效初始值：无参数、`null` 和 `undefined` 都无法推断未来值的预期类型。",
    "显式类型参数描述字符串和可空 User refs。`ref(0)` 已有具体数字初始值，可以依靠推断。",
  ],
  "script/require-valid-default-prop": [
    "要求 prop 默认值符合声明类型",
    "Number 和 Boolean props 使用了不匹配的标量默认值，Array 和 Object props 使用共享字面量而不是工厂。",
    "标量默认值改为 `0` 和 `false`；数组和对象默认值改为返回新值的函数。`[String, Number]` 示例接受字符串默认值，因为它匹配一个声明类型。",
  ],
  "script/return-in-computed-property": [
    "要求每个计算 getter 返回值",
    "块函数体计算 getter 求值 `1 + 2` 却从不返回，使计算值为 undefined。",
    "`return 1 + 2` 将表达式转为 getter 的返回值。规则查找 getter 自身返回值的 return，而非仅有表达式语句。",
  ],
  "script/return-in-emits-validator": [
    "要求每个 Options API emits 校验函数返回值",
    "`submit` 校验函数记录载荷，却不返回校验结果，因此块函数体产生 undefined。",
    "`return payload != null` 为提交载荷提供布尔校验结果，避免结束时没有返回值。",
  ],
  "script/valid-define-emits": [
    "要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）",
    '同一 `defineEmits` 调用同时提供类型参数和运行时数组 `["save"]`，混合两种互斥声明。',
    "移除运行时参数，只保留一个针对 `save` 的类型事件声明。",
  ],
  "script/valid-define-options": [
    "要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots）",
    "第一个调用将专用的 `props` 声明放入 `defineOptions`；后续调用又重复宏，并包含非对象参数。这些展示了禁止的形式和重复调用限制。",
    "一个 `defineOptions` 调用接收只包含受支持普通选项 `name` 和 `inheritAttrs` 的对象。",
  ],
  "script/valid-define-props": [
    "要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）",
    "同一 `defineProps` 调用同时提供类型参数 `{ title: string }` 和运行时参数 `{ title: String }`，编译器不允许二者组合。",
    "移除运行时对象，只保留一个针对 `title` 的类型声明，不再组合两种声明形式。",
  ],
  "script/valid-next-tick": [
    "要求 nextTick() 调用结果被等待、链式处理，或提供回调",
    "导入的 `nextTick()` 是无回调的独立表达式，其返回 Promise 被忽略，没有操作等待 DOM 刷新。",
    "`await nextTick()` 使用该 Promise，显式等待下一次 DOM 更新，再继续执行后续 setup 代码。",
  ],
  "ssr/no-browser-globals-in-ssr": [
    "禁止 SSR 上下文中的浏览器专用全局变量",
    "setup 立即读取 `window.innerWidth`，但组件在服务器运行时不存在 `window`。",
    "初始宽度是服务端安全的 ref 值，浏览器访问移到 `onMounted`，在客户端运行而不在 SSR setup 期间运行。",
  ],
  "ssr/no-hydration-mismatch": [
    "禁止导致水合不匹配的不确定值",
    "模板渲染时执行 `Math.random()`，因此服务端和客户端可能为同一段落生成不同文本。",
    '段落渲染稳定的 `seed` 状态，不再生成新随机结果。在此 Nuxt 风格示例中，`useState` 提供共享状态，初始值是常量 `"stable"`。',
  ],
  "type/no-floating-promises": [
    "禁止悬空（未处理）的 Promise",
    "异步 `save` 函数返回 Promise，但独立的 `save()` 调用既不等待也不返回它，也未显式标记有意丢弃。",
    "`void save()` 显式标记此规则接受的发出后不等待意图。这是显式丢弃标记，不是拒绝处理器。",
  ],
  "type/no-reactivity-loss": [
    "禁止赋值和调用中对响应式值取普通快照",
    "`const count = state.count` 对响应式属性取普通数字快照，因此之后 `state.count` 的更新不会反映到该绑定。",
    '`toRef(state, "count")` 使 `count` 继续关联原响应式属性，不复制其当前原始类型值。',
  ],
  "type/no-unsafe-template-binding": [
    "禁止解析为不安全类型的模板绑定",
    "插值的 `value` 被显式注解为 `any`，检查器无法为模板绑定提供安全的具体类型。",
    "将注解改为 `string`，为同一插值提供可检查的具体类型，而不改变渲染值。",
  ],
  "type/require-typed-emits": [
    "要求 defineEmits 具有类型定义",
    '仅数组形式的 `defineEmits(["save"])` 声明事件名称，却没有类型化载荷约定。',
    "`defineEmits<{ save: [] }>()` 以空载荷元组声明带类型的 `save` 事件，明确表示不接收载荷参数。",
  ],
  "type/require-typed-props": [
    "要求 defineProps 具有类型定义",
    '仅数组形式的 `defineProps(["title"])` 只按名称声明 `title`，没有提供类型。',
    "`defineProps<{ title: string }>()` 为 `title` 提供显式字符串类型，替代仅名称的运行时声明。",
  ],
  "type/strict-boolean-expressions": [
    "要求脚本和模板条件使用安全的布尔表达式",
    "`if (count)` 依赖可空数字绑定的真值性，而非显式布尔检查，也将零与缺失混为一谈。",
    "`count !== undefined && count > 0` 分别检查存在性和正值，缩窄可选值后产生显式布尔条件。",
  ],
  "vapor/no-inline-template": [
    "禁止已弃用的 inline-template 属性",
    "LegacyCard 为子标记使用 inline-template 属性。",
    "标记通过默认插槽传入，而非内联模板。",
  ],
};
