export const chineseVue1: Record<string, readonly [purpose: string, bad: string, good: string]> = {
  "vue/no-mutating-props": [
    "禁止修改组件 props",
    "递增 props.count 会直接写入父组件提供的值。",
    "组件发出携带下一个值的 update:count 事件，由父组件负责更新 prop。",
  ],
  "vue/no-negated-v-if-condition": [
    "禁止在含有 v-else 的条件链中使用否定的 v-if 条件",
    "成对的 v-if 和 v-else 分支以否定条件开头。",
    "先使用肯定条件 ok；反转条件时，将原来的相反分支放在前面。单独使用否定的 v-if 和 !== 比较仍被允许。",
  ],
  "vue/no-non-component-keep-alive-child": [
    "禁止在 `<KeepAlive>` 的直属子层使用普通元素包装",
    "KeepAlive 有条件地包装原生 div，而没有直接缓存 UserCard。",
    "第一个示例将 UserCard 设为条件子节点。v-show 包装层展示了不属于此条件子节点检查范围的结构，并不保证原生包装元素会被缓存。",
  ],
  "vue/no-preprocessor-lang": [
    "不建议使用 CSS 预处理器，优先使用现代 CSS",
    "style 块通过 lang 选择 SCSS。这描述了不使用预处理器的预期规范；当前 SFC 路径不会报告此规则。",
    "相同的 CSS 声明省略了预处理器 lang。这修正了规范上的问题，但目前不会产生可执行的 Bad/Good 诊断差异。",
  ],
  "vue/no-reserved-component-names": [
    "禁止使用保留名称作为组件名称",
    "组件名称 button 与原生 HTML 元素名称冲突。",
    "AppButton 是应用组件名称，没有复用原生 button 名称。",
  ],
  "vue/no-root-v-if": [
    "禁止在模板的唯一根元素上使用 v-if",
    "组件根元素本身会在 v-if 控制下出现或消失。",
    "稳定的外层 div 始终作为根元素，而内层段落承担可见性条件。",
  ],
  "vue/no-script-non-standard-lang": [
    "不建议使用非标准的 script lang 值",
    "脚本在 lang=coffee 下使用 CoffeeScript 语法。当前 SFC 路径不会针对该语言报告此目录规则。",
    "脚本在 lang=ts 下使用普通 TypeScript 声明，展示预期的语言规范。",
  ],
  "vue/no-src-attribute": [
    "不建议在 SFC 块上使用 src 属性",
    "SFC 块通过 src 文件提供模板、脚本和样式内容。",
    "每个 SFC 块都包含自身内容，不使用外部 src 属性。",
  ],
  "vue/no-static-inline-styles": [
    "禁止静态内联 style 属性",
    "段落在 style 属性中包含固定的颜色声明。",
    "notice 类和 scoped 样式表将固定颜色放在模板属性之外。",
  ],
  "vue/no-template-key": [
    "禁止在 `<template>` 上使用 `key` 属性",
    "非循环的 template 包装层具有 key，但它并不是需要 key 的迭代边界。",
    "key 属于 template v-for 迭代，用来标识每个重复的片段。",
  ],
  "vue/no-template-lang": [
    "不建议在 template 块上使用 lang 属性",
    "模板通过 lang 选择 Pug。这是只使用 HTML 的预期规范；当前 SFC 路径不会针对该目录 ID 产生诊断。",
    "普通 HTML 模板省略 lang，直接使用段落。这展示了规范，并未声称当前 SFC 会报告问题。",
  ],
  "vue/no-template-shadow": [
    "禁止使用遮蔽外层作用域变量的变量名称",
    "内层 v-for 再次声明 item，在嵌套循环中遮蔽了外层 item 绑定。",
    "内层循环声明 child，外层行仍使用 item，内层行则使用 child。",
  ],
  "vue/no-template-target-blank": [
    '禁止使用 target="_blank" 而不设置 rel="noopener noreferrer"',
    "外部链接打开新的浏览上下文，却没有预期的 rel 保护。",
    "同一链接在 target=_blank 之外还包含 noopener noreferrer。",
  ],
  "vue/no-textarea-mustache": [
    "禁止在 `<textarea>` 内使用双花括号插值",
    "textarea 将 message 写在子内容插值中，而没有绑定其值。",
    "v-model 将可编辑的 textarea 值绑定到 message。",
  ],
  "vue/no-undefined-refs": [
    "禁止在模板中引用未定义的变量",
    "模板读取 missing，但脚本只声明了 message。",
    "插值读取已存在的 message 绑定。",
  ],
  "vue/no-unsafe-url": [
    "警告可能不安全的 URL 绑定",
    "锚点的目标地址以可执行的 javascript: 协议开头。",
    "锚点使用普通的本地导航目标 /next。",
  ],
  "vue/no-unsandboxed-iframe": [
    "要求 iframe 元素具有 sandbox 属性",
    "嵌入的框架没有使用 sandbox 属性限制其能力。",
    "sandbox 施加限制；需要脚本能力时，allow-scripts 显式允许这一项能力。",
  ],
  "vue/no-unused-components": [
    "禁止注册未在模板中使用的组件",
    "UserAvatar 作为组件导入，但模板从未渲染它。",
    "模板渲染导入的 UserAvatar，并传入 user 绑定。",
  ],
  "vue/no-unused-properties": [
    "禁止 defineProps 中定义的未使用属性",
    "组件将 description 声明为 prop，但只渲染了 title。",
    "模板引用了两个已声明的 props。",
  ],
  "vue/no-unused-refs": [
    '报告从未在 &lt;script&gt; 中引用的模板 ref（ref="x"）',
    "模板声明了名为 unused 的 ref，但脚本中没有对应的引用绑定。",
    "模板 ref inputEl 在 script setup 中具有同名的 ref 绑定。",
  ],
  "vue/no-unused-setup-bindings": [
    "禁止从未读取的 script setup 绑定",
    "script setup 中的 message 绑定从未被模板读取。",
    "段落插值使用 message，从而使用了声明的绑定。",
  ],
  "vue/no-unused-vars": [
    "禁止 v-for 和 v-slot 指令中未使用的变量定义",
    "循环声明了未使用的 index，插槽也声明了未被引用的 foo。",
    "示例使用 index，或将其命名为 _index 以标记有意不使用；插槽则渲染 data。这里的索引 key 只展示变量使用，并不建议用它保持列表项身份稳定。",
  ],
  "vue/no-use-v-else-with-v-for": [
    "禁止在同一元素上使用 `v-else-if` 或 `v-else` 与 `v-for`",
    "else 分支和 v-for 迭代都附加在同一段落上。",
    "独立的 template 承担 v-else，其子段落承担 v-for。",
  ],
  "vue/no-use-v-if-with-v-for": [
    "禁止在同一元素上使用 `v-if` 与 `v-for`",
    "同一个列表元素同时使用 v-if 和 v-for，并通过循环绑定检查可见性。",
    "计算属性中的集合先筛选出可见项，再由模板进行迭代。",
  ],
  "vue/no-useless-mustaches": [
    "禁止表达式为常量字符串字面量的双花括号插值",
    "插值仅包含常量字符串，无需对表达式求值。",
    "字面文本直接写入模板；变量表达式、带插值的模板字符串，以及有意作为分隔符的空白，仍可使用插值。",
  ],
  "vue/no-useless-template-attributes": [
    "禁止在 `<template>` 元素上使用不起作用的属性",
    "条件 template 具有 class，但这个结构性包装层不会渲染可接收该属性的 DOM 元素。",
    "class 移到实际渲染的段落上，v-if 则保留在结构性 template 上。",
  ],
  "vue/no-useless-v-bind": [
    "禁止值为普通字符串字面量的 v-bind",
    "foo 绑定对带引号的常量字符串，或不含插值的模板字符串，进行求值。",
    "常量值改为静态属性；变量值和含插值的值保留绑定。",
  ],
  "vue/no-v-for-template-key-on-child": [
    "禁止在 `<template v-for>` 的子节点上使用 `key`",
    "子段落具有 key，但 template 迭代自身没有 key。",
    "key 移到 template v-for 上，用来标识整个重复片段。",
  ],
  "vue/no-v-html": [
    "警告使用 v-html，以避免 XSS 漏洞",
    "v-html 将 content 解释为 HTML，而不是普通文本。",
    "双花括号插值将 content 显示为转义后的文本，而不注入 HTML。",
  ],
  "vue/no-v-text-v-html-on-component": [
    "禁止在组件元素上使用 v-text / v-html",
    "组件标签使用 v-html 或 v-text，这些指令会替换元素内容，而不是提供组件插槽。",
    "原生 HTML 目标可接收这些指令；MyComponent 通过默认插槽接收内容。",
  ],
  "vue/no-v-text": [
    "禁止 v-text 指令，优先使用双花括号插值",
    "div 的内容通过 v-text 指令提供。",
    "双花括号插值直接在元素内容中表达相同的文本绑定。",
  ],
  "vue/permitted-contents": [
    "要求符合 HTML 内容模型规则",
    "示例在 p 中放置块级内容、省略表格主体、嵌套交互控件，或将 div 直接放在 ul 内。",
    "示例使用段落内的行内内容、显式 tbody 和 li 子节点。自定义 MyItem 不被视为已知的原生 ul 子节点。",
  ],
  "vue/prefer-props-shorthand": [
    "建议使用 props 简写语法（Vue 3.4+）",
    "每个绑定都重复了对应的变量名，包括与连字符参数对应的驼峰名称。",
    "Vue 3.4+ 的同名绑定简写省略了重复表达式；bar 等不同的来源变量仍显式写出。",
  ],
  "vue/prefer-true-attribute-shorthand": [
    "优先使用绑定到 `true` 的布尔属性简写",
    "原生布尔属性 disabled 绑定了常量 true。",
    "原生属性使用布尔简写。false 绑定和组件 props 保留显式值。",
  ],
  "vue/prop-name-casing": [
    "规范所声明 prop 名称的大小写风格",
    "声明的 prop 名称 user_name 使用下划线分隔。",
    "声明及其模板引用都使用驼峰名称 userName。",
  ],
};
