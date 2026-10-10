export const chineseRules5: Record<string, readonly [string, string, string]> = {
  "vize:croquis/cf/non-unique-id": [
    "循环内的元素 id 对每个列表项并不唯一。",
    "每次 `v-for` 迭代都渲染相同的 `result-title` 字面量 ID；循环的 key 不会让 DOM ID 唯一。",
    "标题 ID 包含结果的稳定 ID，为每个列表项产生不同的文档标识符。",
  ],
  "vize:croquis/cf/object-identity-comparison": [
    "响应式对象按身份比较，解包后身份会改变。",
    "`proxy === raw` 比较包装对象身份，因此即使二者代表同一用户记录，结果也是 false。应用希望比较记录身份，而非包装对象身份。",
    "比较稳定的记录 `id`，回答预期问题，不依赖对象是原始对象还是代理。",
  ],
  "vize:croquis/cf/pinia-getter": [
    "Pinia getter 未通过 `storeToRefs` 读取，无法保持响应性。",
    "`const doubled = store.doubled` 在 setup 时复制 getter 的当前数字。复制的数字不会跟随后续 `store.count` 更新。",
    "`storeToRefs(store)` 提供响应式 getter ref，可被解构并由模板解包，同时继续关联 store。",
  ],
  "vize:croquis/cf/prop-type-mismatch": [
    "传入的 prop 值与声明类型不匹配。",
    "父组件将数字表达式 `42` 传给已解析子组件的 `title: string` prop。",
    '字面量 `title="Hello"` 提供符合子组件声明的字符串。',
  ],
  "vize:croquis/cf/provide-inject-type": [
    "提供值与其 inject 类型不一致。",
    "提供者将 `title` 显式注解为 `string`，后代却为同一键请求 `inject<number>`。",
    "消费者显式的 `inject<string>` 与提供者注解一致。保留 `as string`：此诊断产生逻辑比较显式注解，而非推断的字面量类型。",
  ],
  "vize:croquis/cf/provide-without-symbol": [
    "`provide` 使用普通键，而不是 `InjectionKey` symbol。",
    '两个组件都使用字符串 `"theme"`，无关功能可能意外复用此键。',
    "导出一个带类型的 `ThemeKey` symbol，并在 provide 和 inject 处导入同一值。创建描述相同但独立的 symbols 并不能建立联系。",
  ],
  "vize:croquis/cf/reactive-export": [
    "响应式状态从模块导出。",
    "模块导出一个已初始化的响应式对象，所有导入者都收到同一个 count。在请求间共享的 SSR 模块中，这违背示例要求的实例与请求状态隔离。",
    "模块导出工厂，由 App 在 setup 内调用。每个实例获得新的响应式 count，而非导出的单例。",
  ],
  "vize:croquis/cf/reactivity-outside-setup": [
    "响应式 API 在 `setup` 外被调用。",
    "两个响应式 API 都在模块加载时运行，因此两个 Counter 实例共享一个 ref 和计算值，违背独立计数器的预期。",
    "`useCounter` 在每次组件 setup 调用中同步创建 ref 和计算值，为每个控件提供自己的状态及受跟踪的派生值。",
  ],
  "vize:croquis/cf/reassignment-breaks-reactivity": [
    "重新赋值响应式绑定，将其替换为普通值。",
    "子组件创建 prop ref 后用 `props.user` 覆盖变量，丢弃该 ref 联系。",
    "将 `toRef` 保留在 `const` 绑定中，移除替换它的重新赋值。",
  ],
  "vize:croquis/cf/reference-escapes-scope": [
    "响应式引用逸出拥有其生命周期的作用域。",
    "进程级缓存保留组件的活 count ref，可能让实例状态在卸载后仍可达，并观察后续编辑，但此缓存原本用于存储快照。",
    "缓存接收当前普通数字，保留快照而不保留组件拥有的 ref。",
  ],
  "vize:croquis/cf/setup-context-violation": [
    "setup 上下文被以 Vue 不允许的方式使用。",
    "`ref(0)` 在普通脚本模块作用域创建，位于此分析器情形所表示的按实例 setup 上下文之外。",
    "将绑定移入 script setup，使每个组件实例拥有自己的 count，模板也能读取它。",
  ],
  "vize:croquis/cf/shallow-deep-access": [
    "读取 `shallowReactive` 或 `shallowRef` 的深层属性，却假定它受到跟踪。",
    "`shallowReactive` 跟踪根 `user` 属性，但嵌套对象仍是原始对象。修改 `profile.user.name` 不会作为受跟踪的深层修改通知模板。",
    "深层 `reactive` 包装嵌套 user 对象，使同样的名称赋值能触发显示名称更新。",
  ],
  "vize:croquis/cf/spread-breaks-reactivity": [
    "展开响应式对象会复制值并丢失跟踪。",
    "`UserSummary.vue` 将 `props.user` 展开为新对象，取得传入响应式数据的快照。",
    '`toRef(props, "user")` 保留对传入 prop 的引用，不复制其字段。',
  ],
  "vize:croquis/cf/suspense-no-fallback": [
    "`<Suspense>` 没有后备内容。",
    "Suspense 边界有异步子组件，却没有后备内容，因此此示例的等待状态没有加载内容。",
    "`#fallback` 插槽在异步子组件完成前提供显式的加载段落。",
  ],
  "vize:croquis/cf/template-ref-timing": [
    "模板 ref 在组件挂载之前被读取。",
    "setup 在挂载前读取模板 ref，此时值仍为 null，因此可选的 focus 调用不会执行聚焦操作。",
    "`onMounted` 将读取延迟到 Vue 为模板 ref 赋入输入元素之后，使聚焦辅助函数能作用于该元素。",
  ],
  "vize:croquis/cf/toraw-mutation": [
    "使用 `toRaw` 后修改了原始对象。",
    "`rename` 获取原始目标并写入 `raw.name`，绕过本应通知显示名称更新的代理 setter。",
    "通过传入的响应式代理写入 `profile.name`，保留相同重命名行为，同时通知依赖项。",
  ],
  "vize:croquis/cf/uncaught-error": [
    "组件可能抛错，却没有错误边界捕获。",
    "子组件模板对格式错误的输入调用 `JSON.parse`，可达父组件没有错误捕获边界。",
    "父组件为该子组件注册 `onErrorCaptured`。返回 `false` 会停止传播；生产环境边界还应提供有用的恢复界面。",
  ],
  "vize:croquis/cf/undeclared-emit": [
    "组件发出了未声明的事件。",
    '子组件调用 `emit("save")`，但 `defineEmits` 约定只声明了 `cancel`。',
    "以空参数元组声明 `save`，使发出的事件符合组件约定。",
  ],
  "vize:croquis/cf/undeclared-prop": [
    "父组件传入了子组件未声明的 prop。",
    "父组件传入 `typo`，但已解析子组件只声明 `title`。此分析器规范独立于 Vue 通常的属性透传行为。",
    "移除意外的 `typo` 绑定，保留已声明的 `title` prop。",
  ],
  "vize:croquis/cf/undefined-slot": [
    "父组件填充了子组件未暴露的插槽。",
    "App 提供 `footer` 插槽，但 Card 仅声明和渲染 `header`，传入的 Notice 内容在此子组件中没有匹配的插槽出口。",
    "App 提供 `header`，与子组件类型化插槽声明和渲染出口一致，因此 Notice 在该处显示。",
  ],
  "vize:croquis/cf/unhandled-event": [
    "子组件发出的事件没有父组件处理。",
    "`Child.vue` 发出 `save`，直属包装组件却没有监听；组件事件不会自动穿过包装组件冒泡。",
    "`Wrapper.vue` 为直接子组件附加 `save` 监听器。空回调展示此规则认可的处理形式，并不是完整保存实现。",
  ],
  "vize:croquis/cf/unmatched-inject": [
    "`inject` 指定的键没有任何祖先提供。",
    "`ThemeLabel.vue` 注入 `ThemeKey`，但其可达祖先 `App.vue` 从未提供该键。",
    "`App.vue` 在渲染注入的后代之前，使用同一导出的 `ThemeKey` 提供响应式 theme。",
  ],
  "vize:croquis/cf/unmatched-listener": [
    "父组件监听了子组件不发出的事件。",
    "父组件监听 `save`，已解析子组件却只声明 `cancel`。",
    "子组件声明并发出 `save`，与父组件监听器名称一致。",
  ],
  "vize:croquis/cf/unregistered-component": [
    "模板使用了未注册或导入的组件。",
    "虽然存在 `Child.vue` 文件，父组件既没有导入它，也没有为模板以其他方式注册 `Child`。",
    "在父组件 script setup 中导入 `Child`，使模板能解析组件绑定。",
  ],
  "vize:croquis/cf/unresolved-import": [
    "导入无法解析为模块。",
    "父组件导入 `./Missing.vue`，但项目包含的是 `Child.vue`，并非该路径。",
    "将导入指向已存在的 `./Child.vue` 文件，保留相同模板绑定。",
  ],
  "vize:croquis/cf/unused-attrs": [
    "透传属性传给了未使用它们的多根节点组件。",
    "父组件的 `tracking-code` 既未被多根节点子组件作为 prop 使用，也未被转发。",
    "在 `<main>` 上绑定 `$attrs`，为该透传属性提供显式目标。",
  ],
  "vize:croquis/cf/unused-emit": [
    "已声明的 emit 从未使用。",
    "子组件声明了 `save`，却从未以该名称调用事件发出函数。",
    '示例调用 `emit("save")`，使声明事件得到使用。真实交互应在对应操作发生时发出它。',
  ],
  "vize:croquis/cf/unused-provide": [
    "提供的键从未被注入。",
    "`App.vue` 提供 `ThemeKey`，但渲染的 `Dashboard.vue` 子树没有该键的消费者。",
    "仪表盘现在渲染 `ThemeLabel.vue`，后者注入与祖先完全相同身份的 `ThemeKey`。",
  ],
  "vize:croquis/cf/value-extraction-breaks-reactivity": [
    "将响应式值读取到局部变量后丢失后续更新。",
    "Vue 3.5 响应式解构的 `item` 被一次性读取到 `itemSnapshot`，后续 prop 替换不会更新该快照。",
    "在 `computed` 内读取 `item`，让 Vue 的响应式 props 解构转换跟踪每次求值。",
  ],
  "vize:croquis/cf/watch-can-be-computed": [
    "侦听器只将值复制到状态，可以改为计算属性。",
    "侦听器没有外部副作用，只将第二个可写 ref 同步为 `count` 的两倍。示例没有对该派生值的独立写入。",
    "计算 getter 直接表达相同派生关系，移除手动同步及额外可写状态。",
  ],
  "vize:croquis/cf/watcheffect-async": [
    "`watchEffect` 启动异步任务，却无法清理上一轮执行。",
    "异步 `watchEffect` 将隐式依赖收集与等待请求混合，且没有失效保护。",
    "显式的 `watch(() => props.query, ...)` 声明来源、注册请求清理，并在失效后拒绝过期响应。",
  ],
  "vize:croquis/cf/watcher-outside-setup": [
    "`watch` 或 `watchEffect` 在 `setup` 外被调用。",
    "侦听器在模块加载时创建，位于两个 Observer 的 setup 之外，两实例共享其 refs。某个 Observer 卸载时不会自动停止它。",
    "每个同步 setup 调用在 `useObserver` 中创建自身 refs 和侦听器。Vue 将该侦听器与调用组件的生命周期关联。",
  ],
  "vue/cross-file-attrs-fallthrough": [
    "父组件传入属性，但已解析子组件的根无法继承它们，也没有显式使用 $attrs。",
    '父组件将 `class="notice"` 传给已解析的片段子组件；该子组件没有自动属性目标，也从不读取 `$attrs`。',
    "子组件将 `$attrs` 绑定在 `<main>` 上，选择它作为目标；同级 `<aside>` 保持独立。",
  ],
};
