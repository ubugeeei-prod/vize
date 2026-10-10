export const chineseVue0: Record<string, readonly [purpose: string, bad: string, good: string]> = {
  "vue/a11y-img-alt": [
    "要求图片提供 alt 属性，以支持无障碍访问",
    "静态图片和动态来源的图片都未提供 alt 属性。",
    "承载信息的图片使用描述性 alt 文本，装饰性图片使用空 alt，动态图片则绑定其描述。",
  ],
  "vue/attribute-hyphenation": [
    "规范自定义组件的属性命名风格",
    "组件属性使用了驼峰形式 firstName。",
    "first-name 符合配置要求的连字符组件属性命名规范。",
  ],
  "vue/attribute-order": [
    "要求属性保持一致的排列顺序",
    "事件处理器位于结构性 v-if 指令和普通 id 属性之前。",
    "按照规则的顺序，先写 v-if，再写 id，最后写事件处理器。",
  ],
  "vue/component-definition-name-casing": [
    "要求组件定义名称使用 PascalCase 或 kebab-case",
    "文件名 myComponent.vue 以小写字母开头，却在内部使用大写字母，既不是 PascalCase，也不是 kebab-case。",
    "将文件重命名为 MyComponent.vue 即采用 PascalCase；模板内容无需更改。",
  ],
  "vue/component-name-in-template-casing": [
    "要求模板中的组件名称使用指定的大小写风格",
    "配置要求 PascalCase，但组件名称使用了 kebab-case 和 camelCase。",
    "MyComponent 使用 PascalCase；原生 slot 语法仍使用小写。",
  ],
  "vue/html-button-has-type": [
    "要求 button 元素显式指定有效的 type",
    "一个按钮省略了 type，另一个使用了不受支持的 foo 类型。",
    "按钮指定 button、submit 或 reset；绑定的 type 被视为动态值。",
  ],
  "vue/html-quotes": [
    "规范 HTML 属性的引号风格",
    "属性使用单引号或不加引号，不符合双引号规范。",
    "普通属性和指令表达式都使用双引号。",
  ],
  "vue/html-self-closing": [
    "规范自闭合标签风格",
    "空组件使用成对的起止标签，而空元素 img 和 br 没有采用配置要求的自闭合写法。",
    "组件和空元素使用自闭合语法；包含内容的 div 保留结束标签。",
  ],
  "vue/max-template-complexity": [
    "限制组件自身模板的复杂度（圈复杂度和认知复杂度）",
    "父组件编写的分支、循环、插槽内容和表达式中的判断产生了 13 和 25 的复杂度，超过默认上限 11 和 16。",
    "父组件模板将渲染交给 RowList，只保留一个 v-if；其自身的复杂度为 2 和 1。",
  ],
  "vue/multi-word-component-names": [
    "要求组件名称由多个单词组成",
    "Item.vue 使用了单个单词的组件名称。",
    "TodoItem.vue 为同一模板使用了多个单词组成的组件名称。",
  ],
  "vue/mustache-interpolation-spacing": [
    "要求双花括号插值内部的空格保持一致",
    "文本插值在一个或两个分隔符边界处缺少空格。",
    "表达式与双花括号的起止分隔符之间均有空格。",
  ],
  "vue/no-array-index-key": [
    "禁止直接将 v-for 的索引变量用作 :key",
    "列表使用当前索引作为 key，因此重新排列列表时，列表项的身份会发生变化。",
    "key 来自 item.id，即使位置改变，也能保持每个列表项的身份。",
  ],
  "vue/no-bare-strings-in-template": [
    "禁止在模板中直接写入应当国际化的用户可读文本",
    "可见文本和提供名称的属性直接在模板中嵌入了未翻译的字符串。",
    "可翻译内容调用 $t；示例中的标点和纯数字属于允许的例外。",
  ],
  "vue/no-boolean-attr-value": [
    "禁止为 HTML 布尔属性显式指定值",
    "布尔属性 disabled 和 checked 包含了多余的字符串值。",
    "只要存在这些布尔属性，就能表达相同的启用状态，无需指定值。",
  ],
  "vue/no-child-content": [
    "禁止在使用 v-html 或 v-text 时提供子内容",
    "v-text 会替换段落内容，因此编写的后备文本无法在该指令生效后保留。",
    "移除子文本后，v-text 成为段落内容的唯一来源。",
  ],
  "vue/no-deprecated-filter": [
    "禁止使用已弃用的 Vue 2 管道运算符过滤器语法",
    "管道使用了已移除的 Vue 过滤器语法来应用 capitalize。",
    "调用 capitalize(message)，以普通表达式完成转换。",
  ],
  "vue/no-deprecated-functional-template": [
    "禁止在 SFC 的 `<template>` 上使用 `functional` 属性",
    "SFC 模板使用了已移除的 functional 属性，并读取旧的 props 上下文。",
    "普通模板省略 functional，直接读取组件绑定 msg。",
  ],
  "vue/no-deprecated-html-element-is": [
    "禁止在原生 HTML 元素上使用已弃用的 `is` 属性",
    "原生 div 使用了没有前缀的旧 is 属性来指定 Vue 组件。",
    "动态组件使用 :is；原生元素的写法则显式使用 vue: 前缀。",
  ],
  "vue/no-deprecated-inline-template": [
    "禁止使用已弃用的 `inline-template` 属性",
    "Card 使用已弃用的 inline-template 属性接收其内容。",
    "省略 inline-template 属性，以通常方式传入相同内容。",
  ],
  "vue/no-deprecated-router-link-tag-prop": [
    "禁止在 &lt;router-link&gt; 上使用 `tag` prop",
    "RouterLink 使用了已移除的 tag prop 来指定 button 元素。",
    "插槽将 navigate 提供给显式编写的 button。",
  ],
  "vue/no-deprecated-scope-attribute": [
    "禁止在 &lt;template&gt; 上使用已弃用的 `scope` 属性",
    "插槽模板通过已弃用的 scope 属性声明 props。",
    "默认插槽指令使用当前插槽语法声明相同的 props 绑定。",
  ],
  "vue/no-deprecated-slot-attribute": [
    "禁止使用已弃用的 `slot` 属性",
    "通过旧 slot 属性选择 header 插槽。",
    "v-slot:header 使用当前指令显式选择 header 插槽。",
  ],
  "vue/no-deprecated-slot-scope-attribute": [
    "禁止使用已弃用的 `slot-scope` 属性",
    "模板通过已弃用的 slot-scope 属性接收插槽 props。",
    "#default 指令接收这些 props，无需使用 slot-scope。",
  ],
  "vue/no-deprecated-v-bind-sync": [
    "禁止在 `v-bind` 上使用已弃用的 `.sync` 修饰符",
    "绑定使用了已移除的 .sync 修饰符，包括它与 .camel 的组合。",
    "使用普通的单向 title 绑定；需要更新通道时，则使用 v-model:title。",
  ],
  "vue/no-deprecated-v-on-native-modifier": [
    "禁止在 `v-on` 上使用已弃用的 `.native` 修饰符",
    "组件事件处理器使用了已移除的 .native 事件修饰符。",
    "处理器省略 .native，并保留 .stop 等其他事件修饰符。",
  ],
  "vue/no-deprecated-v-on-number-modifiers": [
    "禁止在 `v-on` 上使用已弃用的数字 `keyCode` 修饰符",
    "键盘事件处理器使用已移除的数字代码 13 和 27 来识别按键。",
    "处理器使用具名按键修饰符 enter 和 esc。",
  ],
  "vue/no-dupe-v-else-if": [
    "禁止在 `v-if` / `v-else-if` 链中使用重复条件",
    "else-if 重复了首个分支已检查的 ready 条件，导致后面的分支无法执行。",
    "第二个分支检查 loading，这是不同的状态，因此能够进入 else-if。",
  ],
  "vue/no-duplicate-attributes": [
    "禁止在同一元素上重复声明属性",
    "同一个按钮声明了两次 class，而不是将它们合并为一个 class 值。",
    "两个类名都放在同一个 class 属性中。",
  ],
  "vue/no-empty-component-block": [
    "禁止空的 SFC 块",
    "template、script 和 style 块都没有实质内容。",
    "每个保留的块都包含实际的标记、脚本声明或样式声明。",
  ],
  "vue/no-inline-style": [
    "不建议使用内联 style 属性",
    "静态 style 属性将颜色声明直接嵌入元素。",
    "类名表达固定颜色；依赖 ratio 的宽度仍使用动态样式绑定，不属于静态属性检查的范围。",
  ],
  "vue/no-invalid-html-attribute": [
    "禁止 HTML 属性使用无效的静态值",
    "锚点元素将 stylesheet 用作 rel 值，但该值应当用于样式表 link 元素。",
    "锚点使用 help，这是适用于所链接帮助资源的 rel 值。",
  ],
  "vue/no-lone-template": [
    "禁止不必要的 `<template>` 元素",
    "内层 template 没有指令或插槽作用，因此不具备结构上的用途。",
    "移除不必要的包装层后，段落直接位于 div 内。",
  ],
  "vue/no-multi-spaces": [
    "禁止连续多个空格",
    "属性之间，或元素名称与第一个属性之间，使用了两个空格。",
    "同样的属性之间使用单个空格。",
  ],
  "vue/no-multiple-objects-in-class": [
    "禁止在 :class 数组绑定中使用多个对象字面量",
    "class 数组包含两个可以合并的顶层对象字面量。",
    "一个对象包含所有类名条件；只有一个对象和一个字符串的数组，以及包含非字面量项的数组，仍被允许。",
  ],
  "vue/no-multiple-template-root": [
    "禁止模板包含多个根节点",
    "启用的单根节点规范在模板根部发现了两个同级段落。",
    "使用 section 将段落包装为一个根节点；仅在需要单根节点约定时启用此规范。",
  ],
};
