export const chineseVue2: Record<string, readonly [purpose: string, bad: string, good: string]> = {
  "vue/require-component-is": [
    "要求 `<component>` 元素具有 `v-bind:is`",
    "动态 `<component>` 没有 `is` 目标，因此 Vue 无法选择要渲染的组件。",
    '`:is="currentComponent"` 提供组件选择；该绑定可在运行时变化。',
  ],
  "vue/require-component-registration": [
    "要求显式导入或注册组件",
    "`MissingWidget` 既未注册，也不在配置的全局组件允许列表中。",
    "`MyButton` 列在示例的 `globals` 选项中。该选项让已知的全局组件免于此检查，并不会注册或导入组件。",
  ],
  "vue/require-scoped-style": [
    "要求 style 标签具有 scoped 属性",
    "`.button` 样式没有作用域限制，可能影响组件外部匹配的元素。",
    "添加 `scoped`，为同样的选择器和声明应用 Vue 的组件作用域。",
  ],
  "vue/require-toggle-inside-transition": [
    "要求 `<transition>` 包裹的元素具有切换条件",
    "`<Transition>` 内的静态子节点没有条件可见性或动态选择，无法触发进入或离开的变化。",
    '`v-if="show"` 改变子节点是否存在，为过渡提供进入和离开的边界。',
  ],
  "vue/require-v-for-key": [
    "要求 `v-for` 指令搭配 `v-bind:key`",
    "每个重复的 `<li>` 都缺少在列表更新时标识对应列表项的 key。",
    '`:key="item.id"` 为每个重复节点提供列表项的身份，而不是当前位置。',
  ],
  "vue/scoped-event-names": [
    "建议使用 context:event 格式的作用域事件名称",
    "`playAudio`、`pauseAudio` 和 `reloadAudio` 使用驼峰后缀表达作用域，不符合规则要求的冒号分隔事件规范。",
    "`audio:play`、`audio:pause` 和 `audio:reload` 共享显式的 `audio:` 作用域。发出事件的组件必须使用相同名称。",
  ],
  "vue/sfc-element-order": [
    "要求 SFC 顶层元素保持一致的顺序",
    "style 块位于 script 块之前，不符合配置的 SFC 块顺序。",
    "各块按照 script → template → style 排列。项目可通过此规则的类型化选项选择其他顺序。",
  ],
  "vue/single-style-block": [
    "建议只使用一个 style 块",
    "组件将带作用域的 panel 和 title 样式分散在两个 style 块中。",
    "两个选择器都保留在同一个带作用域的 style 块中，满足单块规范，也没有丢弃任何样式。",
  ],
  "vue/slot-name-casing": [
    "要求通过 v-slot 使用的具名插槽采用 kebab-case",
    "具名插槽 `mySlot` 使用驼峰命名，而规则要求连字符名称。",
    "`#my-slot` 使用 kebab-case。将对应的插槽出口重命名为相同名称。",
  ],
  "vue/this-in-template": [
    "禁止在模板表达式中使用 `this.`",
    "模板表达式显式访问 `this.message`、`this.className` 和 `this.handleClick`，但 Vue 已直接暴露这些绑定。",
    "直接使用 `message`、`className` 和 `handleClick`。字面字符串 `'this.is.a.string'` 保持不变，因为它不是成员访问。",
  ],
  "vue/use-unique-element-ids": [
    "要求使用 useId() 生成唯一元素 ID，而不是静态字面量",
    "组件的每个实例都会复用字面量 ID `email`，多个实例同时渲染时，标签可能指向错误的输入框。",
    "`useId()` 生成实例的 `emailId`；将同一值绑定到标签的 `for` 和输入框的 `id`。",
  ],
  "vue/use-v-on-exact": [
    "存在带按键修饰符的处理器时，要求在 `v-on` 上使用 `.exact`",
    "普通点击处理器也可能在 Ctrl-click 时运行，与独立的 `.ctrl` 处理器重叠。",
    "`.exact` 将普通点击处理器限制为未按修饰键的点击；Ctrl 专用处理器仍保持独立。",
  ],
  "vue/v-bind-style": [
    "规范 `v-bind` 指令风格",
    "`v-bind:class` 使用了完整形式，但配置的绑定风格要求冒号简写。",
    "`:class` 使用所需的简写，并保留相同表达式；此规则检查写法，而非值的类型。",
  ],
  "vue/v-on-event-hyphenation": [
    "要求组件上 v-on 中的自定义事件名称使用连字符",
    "自定义组件监听器使用了 `@myEvent`，而不是连字符事件名称。",
    "`@my-event` 符合要求的自定义事件写法。下方展示的原生元素监听器和动态事件参数不在此检查范围内。",
  ],
  "vue/v-on-handler-style": [
    "要求 v-on 处理器写为方法引用或内联函数",
    "处理器将修改操作和多条语句直接写在事件属性中。",
    "使用处理器引用；需要内联逻辑时，使用箭头函数或函数表达式。函数边界使处理器形式明确。",
  ],
  "vue/v-on-style": [
    "规范 `v-on` 指令风格",
    "`v-on:click` 使用完整的事件监听形式，但规则要求简写。",
    "`@click` 使用配置的简写，并保留同一处理器。",
  ],
  "vue/v-slot-style": [
    "规范 `v-slot` 指令风格",
    "组件使用 `#default`，template 使用 `v-slot:header`，与规则针对各上下文要求的风格相反。",
    "组件的默认插槽使用 `v-slot`，template 的具名插槽使用 `#header`。",
  ],
  "vue/valid-attribute-name": [
    "要求属性名称有效",
    '`my"attr` 内的引号使属性名称格式错误。此示例产生解析器的 `parser/template` 诊断，并不保证另有独立的规则诊断。',
    "`my-attr` 是格式正确的属性名称，因此模板解析器可以读取该属性及其值。",
  ],
  "vue/valid-template-root": [
    "要求 `<template>` 根节点符合 Vue 3 片段语义",
    "普通的嵌套 `<template>` 占据模板根部，却没有赋予它渲染作用的指令。",
    "`<div>` 是可渲染的根元素。此示例并未对 Vue 3 片段施加普遍的单根节点限制。",
  ],
  "vue/valid-v-bind": [
    "要求 `v-bind` 指令有效",
    "不带参数的 `v-bind` 缺少对象表达式，空参数形式则缺少属性名称。",
    "提供属性与表达式、绑定对象，或使用 Vue 3.4+ 的同名简写，例如 `:loading`。",
  ],
  "vue/valid-v-cloak": [
    "要求 `v-cloak` 指令有效",
    "`v-cloak` 设置了值、参数或修饰符，但它不接受这些内容。",
    "使用不带值的 `v-cloak`；CSS 可隐藏元素，直到 Vue 挂载后移除此属性。",
  ],
  "vue/valid-v-else": [
    "要求 `v-else` 指令有效",
    "示例为 `v-else` 提供表达式、将其与 `v-if` 组合，或省略了紧邻其前的条件分支。",
    "将不带值的 `v-else` 紧接在对应的 `v-if` 分支之后。",
  ],
  "vue/valid-v-for": [
    "要求 `v-for` 指令有效",
    "循环省略了迭代表达式，或添加了不受支持的 `.stop` 修饰符。",
    "使用 `item in items` 或 `(item, index) of items`，提供完整的迭代表达式及示例中的 key。",
  ],
  "vue/valid-v-html": [
    "要求 `v-html` 指令有效",
    "`v-html` 缺少表达式，或使用了此指令不支持的参数或修饰符。",
    '`v-html="html"` 提供有效表达式。语法有效并不会清理 HTML，也不会让不可信内容变得安全。',
  ],
  "vue/valid-v-if": [
    "要求 `v-if` 指令有效",
    "条件省略了表达式，或在同一节点上将 `v-if` 与 else 指令组合。",
    "每个 `v-if` 都具有 `ready` 或 `count > 0` 等非空条件，且没有不兼容的 else 指令。",
  ],
  "vue/valid-v-memo": [
    "要求 `v-memo` 指令有效",
    "不带值的 `v-memo` 没有为 Vue 提供依赖表达式，无法判断何时复用子树。",
    '`v-memo="[valueA, valueB]"` 提供用于记忆化的依赖数组。',
  ],
  "vue/valid-v-model": [
    "要求 `v-model` 指令有效",
    "原生 `<div>` 不能像表单控件一样使用 `v-model`，输入框上不带值的指令也没有可写的目标表达式。",
    "将 input、select、textarea 或自定义组件绑定到示例中的可写变量。",
  ],
  "vue/valid-v-on": [
    "要求 `v-on` 指令有效",
    "监听器形式缺少事件参数，或缺少所需的处理器或对象表达式。",
    "提供事件及其处理器，或将监听器对象传给无参数的 `v-on`。",
  ],
  "vue/valid-v-once": [
    "要求 `v-once` 指令有效",
    "`v-once` 设置了值、参数或修饰符，但此指令是无值的单次渲染标记。",
    "不带值的 `v-once` 将子树标记为只渲染一次，不使用不受支持的语法。",
  ],
  "vue/valid-v-show": [
    "要求 `v-show` 指令有效",
    "`v-show` 缺少可见性表达式，或被放在 `<template>` 上；该元素没有可改变 display 的 DOM 元素。",
    "将可见性表达式应用于 `<div>` 等实际渲染的元素。",
  ],
  "vue/valid-v-slot": [
    "要求 `v-slot` 指令有效",
    "插槽指令位于原生 `<div>` 上，或与其他默认或具名插槽声明冲突。",
    "在组件自身上声明其默认插槽，或在子 `<template #header>` 上声明其具名插槽。",
  ],
  "vue/valid-v-text": [
    "要求 `v-text` 指令有效",
    "`v-text` 缺少文本表达式，或使用了不受支持的参数或修饰符。",
    '`v-text="msg"` 在语法上有效。独立的 `vue/no-v-text` 风格规则仍可能要求优先使用插值。',
  ],
  "vue/warn-custom-block": [
    "警告 SFC 文件中的自定义块",
    "SFC 包含 `<i18n>` 自定义块，需要普通 template、script 和 style 处理之外的外部集成。",
    "示例使用标准的 template 和 script-setup 块。此可选的可移植性警告并不意味着所有自定义块都是无效 Vue。",
  ],
  "vue/warn-custom-directive": [
    "警告需要注册的自定义指令",
    "`v-focus`、`v-mask` 和 `v-click-outside` 需要项目特定的指令实现，因此被此可选规范标记。",
    "示例使用内置的 `v-if`、`v-model` 和 `v-on`。禁用此策略时，正确注册的自定义指令仍可作为有效的 Vue 使用。",
  ],
};

export const chineseVueNotes: Record<string, string> = {
  "The component filename is checked. PascalCase and kebab-case are accepted; mixed casing is reported.":
    "检查的是组件文件名。接受 PascalCase 和 kebab-case；混合大小写会被报告。",
  "Bad has cyclomatic complexity 13 and cognitive complexity 25 (limits: 11 and 16). Each component is measured separately; only inline HTML templates are supported.":
    "Bad 的圈复杂度为 13，认知复杂度为 25（上限分别为 11 和 16）。每个组件单独计量；仅支持内联 HTML 模板。",
  "See [complexity scoring and component boundaries](../../guide/cross-file-complexity.md) for the contributions behind the example's two scores.":
    "参见[复杂度评分与组件边界](../../guide/cross-file-complexity.md)，了解示例中两项评分的各部分贡献。",
  "The filename is the finding. Rename the same component; changing a child tag does not fix it.":
    "问题在于文件名。需要重命名该组件；更改子标签不能修复此问题。",
  "Enable only for a single-root contract. Vue 3 normally supports fragments.":
    "仅在需要单根节点约定时启用。Vue 3 通常支持片段。",
  "This catalog entry does not currently emit its rule-specific finding through SFC lint. The Bad/Good pair describes the intended convention, not an executable finding. Enabling the ID does not supply the missing SFC check.":
    "此目录条目目前不会通过 SFC lint 报告该规则特有的问题。Bad/Good 对照描述的是预期规范，并非可执行的诊断。启用此 ID 也不会补上缺失的 SFC 检查。",
  "The current check compares nested v-for bindings. It does not report a single v-for binding merely because it shares a script binding's name.":
    "当前检查比较嵌套的 v-for 绑定。单个 v-for 绑定仅与脚本绑定同名时，不会因此被报告。",
  "Checks declared prop names, not the casing of attributes passed to a child.":
    "检查所声明的 prop 名称，而不是传给子组件的属性的大小写风格。",
  "List explicit component names supplied by application plugins or Musea previewSetup. PascalCase and kebab-case spellings are accepted; regular expressions are not interpreted. Options do not enable the rule. Later layers replace the list; an empty list clears inherited names.":
    "列出由应用插件或 Musea previewSetup 提供的明确组件名称。接受 PascalCase 和 kebab-case 写法；不解析正则表达式。设置选项不会启用规则。后面的配置层会替换列表；空列表会清除继承的名称。",
  "Malformed attribute spelling is diagnosed by parser/template before this defensive rule sees an attribute. Bad therefore reports parser/template; it does not promise a separate vue/valid-attribute-name finding.":
    "格式错误的属性写法会先由 parser/template 诊断，之后该防御性规则才能读取属性。因此 Bad 报告的是 parser/template，并不保证另有独立的 vue/valid-attribute-name 问题。",
};
