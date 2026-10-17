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
  "vue/valid-v-else-if": [
    "要求 `v-else-if` 指令有效",
    "示例中的 `v-else-if` 前面没有条件分支、缺少条件，或带有参数或修饰符。",
    "将带条件的 `v-else-if` 紧跟在它所延续的 `v-if` 或 `v-else-if` 分支之后。",
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
  "vue/valid-v-is": [
    "要求 `v-is` 指令有效",
    "`v-is` 带有参数或修饰符、缺少值，或用在组件上而不是原生 HTML 元素上。",
    "在 `<tr>` 等原生元素上使用带值的 `v-is`，指定要在该位置渲染的组件。",
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
  "vue/valid-v-pre": [
    "要求 `v-pre` 指令有效",
    "`v-pre` 设置了参数、修饰符或值，但此指令是跳过编译的无值标记。",
    "不带值的 `v-pre` 让子树跳过编译，其中的插值按原样显示。",
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

export const chineseVueNotes = {
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
  "Unsupported modifiers are reported individually at their names, including on long-form `v-on:click`, dynamic arguments and object listener syntax. Accepted names match `eslint-plugin-vue` 10.9.2: Vue's built-in modifiers, the complete keyboard alias table, any single Unicode code point, and names with a finite decimal integer prefix (`Number.parseInt(name, 10)` semantics). For example, `.13suffix` is accepted. Numeric-key deprecation and `.native` migration belong to separate rules.":
    "不支持的修饰符会逐个在其名称处报告，完整形式的 `v-on:click`、动态参数和对象监听器语法也会检查。接受的名称与 `eslint-plugin-vue` 10.9.2 一致：Vue 内置修饰符、完整的键盘别名表、任意单个 Unicode 码点，以及具有有限十进制整数前缀的名称（按 `Number.parseInt(name, 10)` 的语义解析）。例如，`.13suffix` 是有效名称。数字按键的弃用和 `.native` 的迁移由其他规则检查。",
  "Only `.stop` and `.prevent` allow an omitted handler. `.once`, `.enter` and configured custom names still need a handler. Add custom names under `vue.linter.ruleOptions`:":
    "只有 `.stop` 和 `.prevent` 允许省略处理器。`.once`、`.enter` 和配置的自定义名称仍然需要处理器。在 `vue.linter.ruleOptions` 中添加自定义名称：",
  'This permits `@click.custom="handler"`. Names are exact and case-sensitive; options supplement the built-in names and do not enable a disabled rule. Scoped entries can replace inherited names or reset them with `[]`.':
    '此设置允许 `@click.custom="handler"`。名称按原样匹配且区分大小写；选项补充内置名称，不会启用已禁用的规则。作用域条目可以替换继承的名称，或用 `[]` 清空它们。',
  "The keyboard table is adapted from the MIT-licensed [eslint-plugin-vue keyboard aliases](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json); its source and license are preserved alongside the implementation.":
    "键盘别名表改编自采用 MIT 许可证的 [eslint-plugin-vue keyboard aliases](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json)；原始源码和许可证随实现一同保留。",
  "The rule compares listeners for the same event on a native element. It ignores custom component events unless both listeners use `.native`. Different event names and disjoint keyboard filters do not conflict.":
    "此规则比较原生元素上同一事件的监听器。对于自定义组件事件，只有两个监听器都使用 `.native` 时才进行检查。事件名称不同或按键过滤器没有共同按键时，不会发生冲突。",
  "System modifiers are required keys, so `@keydown.alt.shift` also accepts Ctrl+Alt+Shift. If a second handler uses `@keydown.alt.ctrl.shift`, add `.exact` to the broader handler to keep those actions separate. `.exact` permits its declared system modifiers and rejects additional Ctrl, Shift, Alt or Meta keys.":
    "系统修饰符指定必须按下的按键，因此 `@keydown.alt.shift` 也接受 Ctrl+Alt+Shift。如果另一个处理器使用 `@keydown.alt.ctrl.shift`，请为匹配范围更广的处理器添加 `.exact`，以区分这些操作。`.exact` 允许其声明的系统修饰符，并拒绝额外的 Ctrl、Shift、Alt 或 Meta 键。",
  "Multiple keyboard modifiers are alternatives: `@keydown.up.down` accepts either arrow key. It can overlap `@keydown.ctrl.down` on Ctrl+ArrowDown. The rule reports the directive key that needs `.exact`, excluding its handler expression.":
    "多个键盘修饰符表示备选项：`@keydown.up.down` 接受上、下两个方向键中的任意一个。按下 Ctrl+下方向键时，它可能与 `@keydown.ctrl.down` 重叠。规则报告需要 `.exact` 的指令键，不包括其处理器表达式。",
  'The first decimal-prefix numeric modifier determines the finding. `@keyup.13.27="submit"` reports only `13` and offers `@keyup.enter.27="submit"`; another fix pass can replace `27` with `escape`. `@keyup.13suffix="submit"` also becomes `@keyup.enter="submit"`. Values 0–9 are exempt, so `@keyup.0.13="submit"` is clean under this rule. Unknown values such as `999` report without a fix. The existing Vue 2 and petite-vue exemption remains in effect.':
    '首个名称带十进制整数前缀的数值修饰符决定诊断结果。`@keyup.13.27="submit"` 只报告 `13`，并提供修复结果 `@keyup.enter.27="submit"`；再次修复可以将 `27` 替换为 `escape`。`@keyup.13suffix="submit"` 也会变为 `@keyup.enter="submit"`。0–9 不在检查范围内，因此 `@keyup.0.13="submit"` 不会触发此规则。`999` 等未知值会被报告，但不提供修复。现有的 Vue 2 和 petite-vue 豁免仍然有效。',
  "## Numeric selection and fixes": "## 数值修饰符的选择与修复",
  "## Modifier validation": "## 修饰符验证",
  "Unsupported modifiers are reported individually at the modifier name, including on long-form `v-on:click`, dynamic event arguments, and object listener syntax. The accepted names match `eslint-plugin-vue` 10.9.2: Vue's built-in event modifiers, the complete keyboard alias table, any single Unicode code point, and names with a finite decimal integer prefix (JavaScript `Number.parseInt(name, 10)` semantics). For example, `.13suffix` is accepted by this rule. Numeric-key deprecation and `.native` migration diagnostics belong to separate rules.":
    "不受支持的修饰符会在各自名称的位置被单独报告，包括长格式 `v-on:click`、动态事件参数和监听器对象语法中的修饰符。接受的名称与 `eslint-plugin-vue` 10.9.2 一致：Vue 内置的事件修饰符、完整的按键别名表、任意单个 Unicode 码点，以及带有有限十进制整数前缀的名称（遵循 JavaScript 的 `Number.parseInt(name, 10)` 语义）。例如，此规则接受 `.13suffix`。数字键码的弃用诊断和 `.native` 的迁移诊断由其他规则负责。",
  "Only `.stop` and `.prevent` provide behavior without a handler. A modifier such as `.once`, `.enter`, or a configured custom name still needs a handler expression.":
    "只有 `.stop` 和 `.prevent` 在没有处理器时也能产生行为。`.once`、`.enter` 或配置的自定义名称等修饰符仍然需要处理器表达式。",
  'This allows `@click.custom="handler"`. Names are exact and case-sensitive; options supplement the built-in names and do not enable a disabled rule. Scoped config entries can replace inherited names with a new list or reset them with `[]`.':
    '这允许使用 `@click.custom="handler"`。名称必须精确匹配且区分大小写；这些选项补充内置名称，不会启用已禁用的规则。限定作用范围的配置条目可以用新列表替换继承的名称，或用 `[]` 将其重置。',
  "The keyboard table is adapted from the MIT-licensed [eslint-plugin-vue keyboard aliases](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json). Its source and license are preserved alongside the implementation.":
    "按键表改编自采用 MIT 许可证的 [eslint-plugin-vue 按键别名](https://github.com/vuejs/eslint-plugin-vue/blob/v10.9.2/lib/utils/key-aliases.json)。其源代码和许可证与实现一同保留。",
  "## When handlers overlap": "## 处理器何时会重叠",
  "The check compares `v-for`, `v-slot` (including `#name`), and legacy scoped-slot bindings within the template. It reports the identifier that hides an earlier binding, including declarations on the same element in attribute order. Sibling elements have independent scopes. Property keys and default-value references are not declarations, and `v-pre` makes its subtree inert.":
    "此检查比较模板内的 `v-for`、`v-slot`（包括 `#name`）和旧式作用域插槽绑定。它报告遮蔽先前绑定的标识符，也包括按属性顺序比较的同一元素上的声明。兄弟元素具有独立的作用域。属性键和默认值中的引用不属于声明，`v-pre` 会使其子树不参与检查。",
  "Script and component property names are outside the current check: a single template binding does not produce a warning merely because a script binding has the same name. Diagnostic ranges cover the authored identifier, including escaped spellings and HTML entities.":
    "脚本和组件属性的名称不在当前检查范围内：单个模板绑定不会仅仅因为与脚本绑定同名就产生警告。诊断范围覆盖源代码中实际编写的标识符，包括转义写法和 HTML 实体。",
  "Projects with custom event handling can add names under `vue.linter.ruleOptions`:":
    "使用自定义事件处理的项目可以在 `vue.linter.ruleOptions` 下添加名称：",
} satisfies Record<string, string>;
