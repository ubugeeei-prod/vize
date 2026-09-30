//! `<rule>.description` for script rules, `script/component-options-name-casing`
//! through `script/no-reserved-props`. Registered by
//! [`crate::i18n_rules_markup`], whose module docs state the contract.

/// `(key, en, ja, zh)`.
pub(crate) static ENTRIES: &[(&str, &str, &str, &str)] = &[
    (
        "script/component-options-name-casing.description",
        "Enforce PascalCase for the component `name` option",
        "コンポーネントの `name` オプションに PascalCase を強制する",
        "强制组件的 `name` 选项使用 PascalCase",
    ),
    (
        "script/custom-event-name-casing.description",
        "Enforce camelCase for emitted custom event names",
        "発行するカスタムイベント名に camelCase を強制する",
        "强制触发的自定义事件名使用 camelCase",
    ),
    (
        "script/define-emits-declaration.description",
        "Enforce the type-based defineEmits<{}>() form over the runtime/array form",
        "defineEmits にはランタイム・配列形式ではなく型ベースの defineEmits<{}>() 形式を強制する",
        "强制使用基于类型的 defineEmits<{}>() 形式，而不是运行时/数组形式",
    ),
    (
        "script/define-macros-order.description",
        "Enforce a consistent order of the Vue compiler macros in <script setup>",
        "<script setup> 内の Vue コンパイラーマクロの順序を統一する",
        "强制 <script setup> 中 Vue 编译器宏的顺序保持一致",
    ),
    (
        "script/define-props-declaration.description",
        "Enforce type-based defineProps<{ ... }>() over the runtime/object form",
        "defineProps にはランタイム・オブジェクト形式ではなく型ベースの defineProps<{ ... }>() を強制する",
        "强制使用基于类型的 defineProps<{ ... }>()，而不是运行时/对象形式",
    ),
    (
        "script/define-props-destructuring.description",
        "Disallow destructuring the return value of defineProps in <script setup>",
        "<script setup> で defineProps の戻り値を分割代入することを禁止する",
        "禁止在 <script setup> 中解构 defineProps 的返回值",
    ),
    (
        "script/no-arrow-functions-in-watch.description",
        "Disallow arrow functions as Options API watch handlers",
        "Options API の watch ハンドラーにアロー関数を使うことを禁止する",
        "禁止将箭头函数用作选项式 API 的 watch 处理函数",
    ),
    (
        "script/no-boolean-default.description",
        "Disallow a default on a Boolean prop",
        "Boolean 型の props にデフォルト値を指定することを禁止する",
        "禁止为 Boolean 类型的 prop 设置默认值",
    ),
    (
        "script/no-deprecated-data-object-declaration.description",
        "Disallow an object literal as the component data option (Vue 3 requires a function)",
        "コンポーネントの data オプションにオブジェクトリテラルを使うことを禁止する（Vue 3 では関数が必要）",
        "禁止将对象字面量用作组件的 data 选项（Vue 3 要求使用函数）",
    ),
    (
        "script/no-deprecated-destroyed-lifecycle.description",
        "Disallow deprecated destroyed and beforeDestroy lifecycle hooks",
        "非推奨の destroyed と beforeDestroy ライフサイクルフックを禁止する",
        "禁止使用已弃用的 destroyed 和 beforeDestroy 生命周期钩子",
    ),
    (
        "script/no-deprecated-dollar-listeners-api.description",
        "Disallow the $listeners instance property removed in Vue 3 (merged into $attrs)",
        "Vue 3 で削除された $listeners インスタンスプロパティを禁止する（$attrs に統合）",
        "禁止使用 Vue 3 中已移除的 $listeners 实例属性（已并入 $attrs）",
    ),
    (
        "script/no-deprecated-dollar-scopedslots-api.description",
        "Disallow the $scopedSlots instance property removed in Vue 3 (use $slots)",
        "Vue 3 で削除された $scopedSlots インスタンスプロパティを禁止する（$slots を使用）",
        "禁止使用 Vue 3 中已移除的 $scopedSlots 实例属性（请使用 $slots）",
    ),
    (
        "script/no-deprecated-events-api.description",
        "Disallow the removed Vue 2 events API ($on / $off / $once)",
        "削除された Vue 2 のイベント API（$on / $off / $once）を禁止する",
        "禁止使用已移除的 Vue 2 事件 API（$on / $off / $once）",
    ),
    (
        "script/no-deprecated-props-default-this.description",
        "Disallow `this` inside a prop default/validator function (removed in Vue 3)",
        "props の default・validator 関数の中での `this` を禁止する（Vue 3 で削除）",
        "禁止在 prop 的 default/validator 函数中使用 `this`（Vue 3 已移除）",
    ),
    (
        "script/no-dupe-keys.description",
        "Disallow duplicate keys across Options API props/data/computed/methods/setup/inject",
        "Options API の props・data・computed・methods・setup・inject の間でのキーの重複を禁止する",
        "禁止在选项式 API 的 props/data/computed/methods/setup/inject 之间出现重复的键",
    ),
    (
        "script/no-duplicate-attr-inheritance.description",
        "Flag a component that applies its fallthrough attributes twice",
        "フォールスルー属性を二重に適用しているコンポーネントを検出する",
        "标记重复应用透传属性的组件",
    ),
    (
        "script/no-export-in-script-setup.description",
        "Disallow export statements inside <script setup>",
        "<script setup> 内の export 文を禁止する",
        "禁止在 <script setup> 中使用 export 语句",
    ),
    (
        "script/no-import-compiler-macros.description",
        "Disallow importing Vue compiler macros that are auto-imported",
        "自動インポートされる Vue コンパイラーマクロをインポートすることを禁止する",
        "禁止导入会被自动导入的 Vue 编译器宏",
    ),
    (
        "script/no-internal-imports.description",
        "Disallow importing from Vue internal modules",
        "Vue の内部モジュールからのインポートを禁止する",
        "禁止从 Vue 内部模块导入",
    ),
    (
        "script/no-multiple-slot-args.description",
        "Disallow passing more than one argument to a scoped-slot function call",
        "スコープ付きスロット関数の呼び出しに 2 つ以上の引数を渡すことを禁止する",
        "禁止向作用域插槽函数调用传入多个参数",
    ),
    (
        "script/no-next-tick.description",
        "Disallow nextTick() usage in Vapor-oriented components",
        "Vapor 向けのコンポーネントで nextTick() を使うことを禁止する",
        "禁止在面向 Vapor 的组件中使用 nextTick()",
    ),
    (
        "script/no-potential-component-option-typo.description",
        "Flag likely typos in Options API component option names",
        "Options API のコンポーネントオプション名にありがちなタイプミスを検出する",
        "标记选项式 API 组件选项名中可能的拼写错误",
    ),
    (
        "script/no-reactive-destructure.description",
        "Disallow destructuring reactive objects which loses reactivity",
        "リアクティビティが失われる、リアクティブオブジェクトの分割代入を禁止する",
        "禁止解构响应式对象（会丢失响应性）",
    ),
    (
        "script/no-ref-as-operand.description",
        "Require ref-bound variables to be accessed via `.value` when used as an operand",
        "ref をオペランドとして使うときは `.value` 経由でアクセスすることを必須にする",
        "要求 ref 变量用作运算数时通过 `.value` 访问",
    ),
    (
        "script/no-required-prop-with-default.description",
        "Disallow a prop that is both required: true and has a default",
        "required: true とデフォルト値の両方を持つ props を禁止する",
        "禁止 prop 同时设置 required: true 和默认值",
    ),
    (
        "script/no-reserved-identifiers.description",
        "Disallow using Vue compiler reserved identifiers",
        "Vue コンパイラーの予約識別子を使うことを禁止する",
        "禁止使用 Vue 编译器保留的标识符",
    ),
    (
        "script/no-reserved-keys.description",
        "Disallow Vue-reserved names as Options API props/data/computed/methods/setup/inject keys",
        "Options API の props・data・computed・methods・setup・inject のキーに Vue の予約名を使うことを禁止する",
        "禁止将 Vue 保留名称用作选项式 API 的 props/data/computed/methods/setup/inject 键",
    ),
    (
        "script/no-reserved-props.description",
        "Disallow reserved names in a component's props declaration",
        "コンポーネントの props 宣言で予約名を使うことを禁止する",
        "禁止在组件的 props 声明中使用保留名称",
    ),
];
