export const chineseRules4: Record<string, readonly [string, string, string]> = {
  "vapor/no-vue-lifecycle-events": [
    "禁止 @vue:xxx 元素生命周期事件（Vapor 不支持）",
    "输入框使用了 @vue:mounted 模板生命周期事件。",
    "onMounted 通过受支持的脚本生命周期钩子访问具名模板引用，并聚焦输入框。",
  ],
  "vapor/prefer-static-class": [
    "字符串字面量优先使用静态 class，而不是动态 class 绑定",
    "class 绑定对常量字符串求值，但类名并不会改变。",
    "静态 class 属性表达相同的 panel 类名，无需绑定。",
  ],
  "vapor/require-vapor-attribute": [
    "建议为 script setup 添加 vapor 属性",
    "script setup 块缺少 Vapor 编译属性。这是预期规范：当前为空的规则回调不会诊断它。",
    "添加 vapor 会选择 Vapor 编译。这展示预期修复，并不表示当前 linter 会报告此目录规则。",
  ],
  "ecosystem/vue-router-extra-param": [
    "路由未声明 tab；Vue Router 会丢弃它。",
    "`user-post` 路径声明 `userId` 和 `postId`，但导航还将未声明的 `tab` 作为路径参数传入。",
    "从 params 移除 `tab`，只保留路由路径中的键。应用需要选择标签页时，应另行使用 query。",
  ],
  "ecosystem/vue-router-missing-param": [
    "缺少必需的 postId；依赖当前路由并不稳健。",
    "导航省略了 `user-post` 路径必需的 `postId`。这是警告，因为 Vue Router 可能从当前路由继承该值。",
    "显式传入 `userId` 和 `postId`，使导航不依赖当前路由的参数状态。",
  ],
  "ecosystem/vue-router-param-type": [
    "postId 不是可重复参数，因此数组无效。",
    '`postId` 是标量路径参数，但导航传入了数组 `["2"]`。',
    '为不可重复的 `postId` 路径段传入标量 `"2"`。',
  ],
  "ecosystem/vue-router-unknown-route": [
    "名称不在完整且已安装的路由器中。",
    "可达且已安装的路由器声明了 `user-post`，但导航使用拼写错误的 `user-posts`。",
    "使用已注册的 `user-post` 名称，并保留两个已声明的路径参数。",
  ],
  "html/cross-component-nesting": [
    "检查导入组件组合后的实际 HTML 嵌套。",
    "父组件的 `<p>` 包含根为 `<div>` 的已解析子组件，组合后形成无效的段落与块级元素嵌套。",
    "改用可容纳子组件块级元素的 `<section>` 容器；子组件保持不变。",
  ],
  "vize:croquis/cf/array-mutation": [
    "通过索引修改数组，无法被该响应式数组跟踪。",
    "在此历史 Vue 2.7 项目中，`items[0] = next` 修改数组却不通知 Vue 2 数组观察器，因此显示的首项不一定更新。",
    "`splice(0, 1, next)` 使用 Vue 2 可观察的数组修改方法，使同样的替换能更新视图。",
  ],
  "vize:croquis/cf/async-boundary": [
    "响应式状态跨越异步边界，可能被观察为过期值。",
    "较慢的旧查询可能在新查询之后完成并覆盖 `result`，因为侦听器没有失效清理。",
    "在等待之前注册清理：中止旧请求并将其 `active` 标志设为失效，只赋值仍有效的响应。",
  ],
  "vize:croquis/cf/async-no-suspense": [
    "异步组件在没有 Suspense 边界的情况下渲染。",
    "子组件具有顶层 await，但父组件未提供 `<Suspense>` 边界。当前源代码解析未提供发出此诊断代码所需的宏事实。",
    "父组件将同一异步子组件包在 `<Suspense>` 中，并提供加载后备内容。这展示规范；当前分析遍对两种源代码形式都不发出该诊断。",
  ],
  "vize:croquis/cf/browser-api-ssr": [
    "在组件可能于服务端渲染的位置使用了浏览器专用 API。",
    "`window.innerWidth` 在 setup 期间运行，但 SSR 环境没有浏览器 `window`。",
    "将 ref 初始化为服务端安全的值，并在客户端挂载后运行的 `onMounted` 中读取 `window`。",
  ],
  "vize:croquis/cf/circular-dep": [
    "组件相互导入，形成循环。",
    "`a.ts` 导入 `b.ts`，后者又导入 `a.ts`。两者都立即读取另一模块尚未初始化的常量来初始化自身常量，导致暂时性死区错误。",
    "两个模块都从独立的 `labels.ts` 读取已初始化前缀，移除循环及立即发生的交叉读取。",
  ],
  "vize:croquis/cf/circular-reactive-dependency": [
    "响应式计算相互依赖，形成循环。",
    "App 拥有并提供 count（A）。CycleView 派生 nextCount（B），随后立即将每个派生值写回同一注入的 count。每次写入再次改变计算输入，形成 A → B → A 更新反馈。下方保留图中的身份代表这两个引用，而非仅名称相同的无关绑定。",
    "移除将 B 写回 A 的侦听器。App 保留 count 的所有权，仅通过显式 Increment 操作改变它；CycleView 读取派生 nextCount，不将结果反馈回去。同样的引用只保留 A → B 依赖。",
  ],
  "vize:croquis/cf/closure-captures-reactive": [
    "闭包捕获了响应式值，无法看到后续更新。",
    "`makeReader` 在创建闭包前复制 `count.value`。计算读取器随后返回初始数字，没有读取响应式依赖。",
    "闭包在调用时读取 `count.value`，使计算 getter 能跟踪 ref，并在递增后更新 `shown`。",
  ],
  "vize:croquis/cf/composable-outside-setup": [
    "组合式函数在 `setup` 外被调用。",
    "导入 `use-title.ts` 时，在组件 setup 激活前注册了 `onMounted`。稍后调用导出函数仅返回模块级 ref，无法补救错失的生命周期所有权。",
    "状态创建和钩子注册都移到 `useTitle`，由 App 在 setup 中同步调用。挂载钩子现在属于该 App 实例。",
  ],
  "vize:croquis/cf/computed-side-effects": [
    "计算 getter 写入状态或执行其他副作用。",
    "求值 `doubled` 会写入 `lastCalculated`，因此读取计算值也会修改独立状态，将副作用与惰性 getter 的读取时机耦合。",
    "getter 只返回派生数字。独立侦听器负责在 `count` 变化时写入 `lastCalculated`，也包括初始值。",
  ],
  "vize:croquis/cf/deep-import": [
    "导入链超过项目允许的深度。",
    "入口让简单值经过 `level-one`、`level-two` 和 `level-three`，对期望浅层公共边界的项目形成不必要的深导入链。",
    "入口使用直接重新导出该值的 `public-api.ts`。消费者保留相同导入名称，导入链变短。",
  ],
  "vize:croquis/cf/destructuring-breaks-reactivity": [
    "解构响应式对象会复制字段并丢失跟踪。",
    "普通解构 `props` 对象会复制当前 `item` 值；这与 Vue 3.5 直接解构 `defineProps()` 不同。",
    '`toRef(props, "item")` 保留与 `props` 属性的联系。',
  ],
  "vize:croquis/cf/di-outside-setup": [
    "`provide` 或 `inject` 在 `setup` 外被调用。",
    "`main.ts` 在没有激活组件实例时调用组件 `provide`。子组件的 `inject` 无法获取预期祖先值，因此使用 `light`。",
    "App 在渲染子组件前从 setup 调用提供者。子组件现在从组件祖先继承 `dark`。",
  ],
  "vize:croquis/cf/dom-access-without-next-tick": [
    "在 Vue 完成更新刷新之前读取 DOM。",
    "点击处理器递增 `count` 后立即读取渲染段落，此时 Vue 尚未刷新计划中的 DOM 更新。`sampled` 可能包含之前的计数。",
    "写入状态后等待 `nextTick()`，让 Vue 先更新段落，再由 `readLabel` 读取文本。",
  ],
  "vize:croquis/cf/duplicate-id": [
    "多个组件使用同一个元素 id。",
    '可达的配送与账单组件都渲染 `id="postal-code"`，使其标签共享不明确的文档目标。',
    "每个组件调用 `useId()`，将自身的值同时绑定给标签和输入框，在不重复字面量 ID 的情况下保留关联。",
  ],
  "vize:croquis/cf/event-listener-leak": [
    "注册了事件监听器，却从未移除。",
    "挂载时添加的 window resize 监听器捕获组件 width ref，但卸载时从不移除。重复挂载可能保留未使用的监听器和状态。",
    "`onUnmounted` 移除挂载时注册的同一个 `resize` 函数，结束该实例外部监听器的生命周期。",
  ],
  "vize:croquis/cf/event-modifier": [
    "事件监听器使用了发出事件不支持的修饰符。",
    "`.stop` 假定子组件自定义 `save` 事件具有原生事件的传播方法，但其载荷不一定是 DOM Event。",
    "从自定义事件监听器移除 `.stop`；需要时在实际 DOM 监听器处理原生传播。",
  ],
  "vize:croquis/cf/hydration-risk": [
    "此诊断代码归类多种响应性问题，包括将 prop 复制到 ref。它并不表示跨文件分析遍会检测所有 Date.now() 表达式。",
    "子组件仅初始化一次 `ref(props.count)`，因此局部 count 不再跟随后续父组件 prop 变化。这是当前的 prop 到 ref 诊断产生逻辑，不是普遍的不确定 SSR 示例。",
    '`toRef(props, "count")` 指向 prop，不再将初始值复制到独立状态。',
  ],
  "vize:croquis/cf/inherit-attrs-unused": [
    "设置了 `inheritAttrs: false`，但组件从不读取属性。",
    '子组件设置 `inheritAttrs: false`，却从不转发父组件的 `class="notice"` 属性。',
    "保留显式继承控制，将 `$attrs` 绑定到预期的 `<main>` 目标。",
  ],
  "vize:croquis/cf/inject-without-symbol": [
    "`inject` 使用普通键，而不是 `InjectionKey` symbol。",
    '消费者注入无类型字符串键 `"theme"`，没有与提供者共享的 symbol 身份。',
    "消费者和提供者导入同一个 `ThemeKey`，不重复字符串名称。",
  ],
  "vize:croquis/cf/injected-async-mutation-race": [
    "注入值被可能发生竞态的异步任务修改。",
    "`CountLoader.vue` 将等待得到的结果直接写入与 `CountSummary.vue` 共享的注入 store，使过期任务影响两个消费者。",
    "加载器取消失效任务，只发出有效结果。提供者通过 `applyLoadedCount` 拥有 store 修改权。",
  ],
  "vize:croquis/cf/lifecycle-outside-setup": [
    "生命周期钩子在 `setup` 外注册。",
    "入口在挂载应用前调用 `installTitle()`，因此 `onMounted` 在没有激活组件 setup 上下文时注册。",
    "从 App 的 setup 同步调用同一辅助函数，将生命周期回调关联到该实例的挂载。",
  ],
  "vize:croquis/cf/lifecycle-without-cleanup": [
    "生命周期钩子启动任务，却从不清理。",
    "挂载时注册 window resize 监听器，卸载时却不移除同一回调。",
    "`onUnmounted` 使用与 `addEventListener` 相同的事件名称和函数身份移除监听器。",
  ],
  "vize:croquis/cf/missing-required-prop": [
    "未传入必需的 prop。",
    "父组件渲染 `<Child />`，没有提供子组件必需的 `title: string` prop。",
    '`title="Hello"` 提供已解析子组件声明的必需 prop。',
  ],
  "vize:croquis/cf/missing-suspense": [
    "异步依赖在 Suspense 边界之外使用。",
    "`AsyncCard` 具有顶层 await，使其 setup 异步，但 App 没有使用 Suspense 边界协调该依赖。",
    "App 将异步子组件包在 `Suspense` 内，在子组件 setup 完成前提供加载后备内容。",
  ],
  "vize:croquis/cf/module-scope-reactive": [
    "响应式状态在模块作用域创建，被所有调用者共享。",
    "模块仅初始化一次 `count`，两个 Counter 实例收到同一个 ref。虽然示例期望独立实例状态，点击一个却会改变两个计数器。",
    "在 `createCounter` 内创建 ref，为每个同步 setup 调用提供独立状态对象，使每个按钮拥有自己的计数器。",
  ],
  "vize:croquis/cf/multi-root-attrs": [
    "多根节点组件接收了属性，却没有放置目标。",
    "子组件具有 `<main>` 和 `<aside>` 两个根节点，Vue 没有可自动接收父组件 class 的唯一根节点。",
    "显式将 `$attrs` 转发到 `<main>`，同时保留第二个根节点。",
  ],
  "vize:croquis/cf/mutated-after-escape": [
    "响应式对象逸出其所有者后被修改。",
    "归档保留传给 `publish` 的同一对象，所有者随后修改其名称，使本应属于历史的记录也变成 Grace。TypeScript 的 Readonly 参数不会复制对象。",
    "发布普通副本，将归档的 Ada 记录与之后对响应式 profile 的编辑分开，维护归档快照策略。",
  ],
  "vize:croquis/cf/non-reactive-provide": [
    "提供的值不具响应性，后代无法看到更新。",
    "`ThemeProvider.vue` 提供普通对象。修改其字段不会为注入消费者建立 Vue 响应式依赖。",
    "提供者将 theme 包装在 `ref` 内；同一个注入引用可以跟踪后续变化。",
  ],
};
