---
title: "类型与脚本规则"
description: "检查 `<script>` 与 `<script setup>`：编译器宏、响应式、props 与 emits，以及使用 TypeScript 类型信息的规则。"
---

# 类型与脚本规则

检查 `<script>` 与 `<script setup>`：编译器宏、响应式、props 与 emits，以及使用 TypeScript 类型信息的规则。

每个规则页面都有会触发诊断的示例、不会触发的示例，以及启用该规则的配置。

## 基础 (39)

`essential` 预设中的正确性规则。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="script-no-arrow-functions-in-watch"></span>[`script/no-arrow-functions-in-watch`](./reference/script-no-arrow-functions-in-watch.md) | 禁止将箭头函数用作 Options API watch 处理器。 |  |
| <span id="script-no-async-in-computed"></span>[`script/no-async-in-computed`](./reference/script-no-async-in-computed.md) | 禁止计算属性中的异步函数。 |  |
| <span id="script-no-computed-properties-in-data"></span>[`script/no-computed-properties-in-data`](./reference/script-no-computed-properties-in-data.md) | 禁止在组件的 data 函数中使用计算属性。 |  |
| <span id="script-no-deprecated-data-object-declaration"></span>[`script/no-deprecated-data-object-declaration`](./reference/script-no-deprecated-data-object-declaration.md) | 禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）。 |  |
| <span id="script-no-deprecated-delete-set"></span>[`script/no-deprecated-delete-set`](./reference/script-no-deprecated-delete-set.md) | 禁止使用 Vue 3 中已移除的 Vue 2 $set 和 $delete。 |  |
| <span id="script-no-deprecated-destroyed-lifecycle"></span>[`script/no-deprecated-destroyed-lifecycle`](./reference/script-no-deprecated-destroyed-lifecycle.md) | 禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子。 | ✓ |
| <span id="script-no-deprecated-dollar-listeners-api"></span>[`script/no-deprecated-dollar-listeners-api`](./reference/script-no-deprecated-dollar-listeners-api.md) | 禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）。 |  |
| <span id="script-no-deprecated-dollar-scopedslots-api"></span>[`script/no-deprecated-dollar-scopedslots-api`](./reference/script-no-deprecated-dollar-scopedslots-api.md) | 禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）。 |  |
| <span id="script-no-deprecated-events-api"></span>[`script/no-deprecated-events-api`](./reference/script-no-deprecated-events-api.md) | 禁止已移除的 Vue 2 事件 API（$on / $off / $once）。 |  |
| <span id="script-no-deprecated-model-definition"></span>[`script/no-deprecated-model-definition`](./reference/script-no-deprecated-model-definition.md) | 禁止使用 Vue 3 中已移除的 Vue 2 model 选项。 |  |
| <span id="script-no-deprecated-props-default-this"></span>[`script/no-deprecated-props-default-this`](./reference/script-no-deprecated-props-default-this.md) | 禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）。 |  |
| <span id="script-no-deprecated-vue-config-keycodes"></span>[`script/no-deprecated-vue-config-keycodes`](./reference/script-no-deprecated-vue-config-keycodes.md) | 禁止使用 Vue 3 中已移除的 Vue.config.keyCodes。 |  |
| <span id="script-no-dupe-keys"></span>[`script/no-dupe-keys`](./reference/script-no-dupe-keys.md) | 禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键。 |  |
| <span id="script-no-duplicate-attr-inheritance"></span>[`script/no-duplicate-attr-inheritance`](./reference/script-no-duplicate-attr-inheritance.md) | 标记重复应用透传属性的组件。 |  |
| <span id="script-no-export-in-script-setup"></span>[`script/no-export-in-script-setup`](./reference/script-no-export-in-script-setup.md) | 禁止 &lt;script setup&gt; 内的 export 语句。 |  |
| <span id="script-no-expose-after-await"></span>[`script/no-expose-after-await`](./reference/script-no-expose-after-await.md) | 禁止在 await 之后调用 expose() 和 defineExpose()。 |  |
| <span id="script-no-import-compiler-macros"></span>[`script/no-import-compiler-macros`](./reference/script-no-import-compiler-macros.md) | 禁止导入自动可用的 Vue 编译器宏。 |  |
| <span id="script-no-internal-imports"></span>[`script/no-internal-imports`](./reference/script-no-internal-imports.md) | 禁止从 Vue 内部模块导入。 |  |
| <span id="script-no-lifecycle-after-await"></span>[`script/no-lifecycle-after-await`](./reference/script-no-lifecycle-after-await.md) | 禁止在 setup() 中 await 之后注册生命周期钩子。 |  |
| <span id="script-no-potential-component-option-typo"></span>[`script/no-potential-component-option-typo`](./reference/script-no-potential-component-option-typo.md) | 标记 Options API 组件选项名称中的疑似拼写错误。 |  |
| <span id="script-no-ref-as-operand"></span>[`script/no-ref-as-operand`](./reference/script-no-ref-as-operand.md) | 要求将 ref 绑定变量用作操作数时通过 `.value` 访问。 |  |
| <span id="script-no-required-prop-with-default"></span>[`script/no-required-prop-with-default`](./reference/script-no-required-prop-with-default.md) | 禁止 prop 同时具有 required: true 和默认值。 |  |
| <span id="script-no-reserved-identifiers"></span>[`script/no-reserved-identifiers`](./reference/script-no-reserved-identifiers.md) | 禁止使用 Vue 编译器保留标识符。 |  |
| <span id="script-no-reserved-keys"></span>[`script/no-reserved-keys`](./reference/script-no-reserved-keys.md) | 禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键。 |  |
| <span id="script-no-reserved-props"></span>[`script/no-reserved-props`](./reference/script-no-reserved-props.md) | 禁止在组件 props 声明中使用保留名称。 |  |
| <span id="script-no-side-effects-in-computed-properties"></span>[`script/no-side-effects-in-computed-properties`](./reference/script-no-side-effects-in-computed-properties.md) | 禁止 Options API 计算 getter 中的副作用。 |  |
| <span id="script-no-use-computed-property-like-method"></span>[`script/no-use-computed-property-like-method`](./reference/script-no-use-computed-property-like-method.md) | 禁止像方法一样调用 Options API 计算属性。 |  |
| <span id="script-no-watch-after-await"></span>[`script/no-watch-after-await`](./reference/script-no-watch-after-await.md) | 禁止在 setup() 中 await 之后注册未保留停止句柄的侦听器。 |  |
| <span id="script-prefer-import-from-vue"></span>[`script/prefer-import-from-vue`](./reference/script-prefer-import-from-vue.md) | 优先从 'vue' 导入，而不是内部包。 | ✓ |
| <span id="script-require-prop-type-constructor"></span>[`script/require-prop-type-constructor`](./reference/script-require-prop-type-constructor.md) | 要求 prop 的 `type` 值是构造器，而不是字符串字面量。 |  |
| <span id="script-require-render-return"></span>[`script/require-render-return`](./reference/script-require-render-return.md) | 要求组件的 render 函数返回一个值。 |  |
| <span id="script-require-slots-as-functions"></span>[`script/require-slots-as-functions`](./reference/script-require-slots-as-functions.md) | 要求将 $slots 的属性作为函数调用。 |  |
| <span id="script-require-valid-default-prop"></span>[`script/require-valid-default-prop`](./reference/script-require-valid-default-prop.md) | 要求 prop 默认值符合声明类型。 |  |
| <span id="script-return-in-computed-property"></span>[`script/return-in-computed-property`](./reference/script-return-in-computed-property.md) | 要求每个计算 getter 返回值。 |  |
| <span id="script-return-in-emits-validator"></span>[`script/return-in-emits-validator`](./reference/script-return-in-emits-validator.md) | 要求每个 Options API emits 校验函数返回值。 |  |
| <span id="script-valid-define-emits"></span>[`script/valid-define-emits`](./reference/script-valid-define-emits.md) | 要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）。 |  |
| <span id="script-valid-define-options"></span>[`script/valid-define-options`](./reference/script-valid-define-options.md) | 要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots）。 |  |
| <span id="script-valid-define-props"></span>[`script/valid-define-props`](./reference/script-valid-define-props.md) | 要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）。 |  |
| <span id="script-valid-next-tick"></span>[`script/valid-next-tick`](./reference/script-valid-next-tick.md) | 要求 nextTick() 调用结果被等待、链式处理，或提供回调。 |  |

## 默认 (7)

由默认预设 `happy-path` 添加。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="script-define-props-destructuring"></span>[`script/define-props-destructuring`](./reference/script-define-props-destructuring.md) | 要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致。 |  |
| <span id="script-no-multiple-slot-args"></span>[`script/no-multiple-slot-args`](./reference/script-no-multiple-slot-args.md) | 禁止向作用域插槽函数调用传入多个参数。 |  |
| <span id="script-no-unstable-nested-components"></span>[`script/no-unstable-nested-components`](./reference/script-no-unstable-nested-components.md) | 禁止在 setup 或 render 函数内定义组件。 |  |
| <span id="script-require-define-component"></span>[`script/require-define-component`](./reference/script-require-define-component.md) | 要求 TypeScript SFC 的默认导出使用 defineComponent()。 |  |
| <span id="script-require-explicit-emits"></span>[`script/require-explicit-emits`](./reference/script-require-explicit-emits.md) | 要求发出的事件在 defineEmits 或 emits 选项中声明。 |  |
| <span id="type-require-typed-emits"></span>[`type/require-typed-emits`](./reference/type-require-typed-emits.md) | 要求 defineEmits 具有类型定义。 |  |
| <span id="type-require-typed-props"></span>[`type/require-typed-props`](./reference/type-require-typed-props.md) | 要求 defineProps 具有类型定义。 |  |

## 更严格的预设 (61)

只在 `opinionated`、`nuxt` 或 `ecosystem` 中。具体是哪些见“预设”列。

| 规则 | 检查内容 | 预设 | 自动修复 |
| --- | --- | --- | --- |
| <span id="script-component-options-name-casing"></span>[`script/component-options-name-casing`](./reference/script-component-options-name-casing.md) | 要求组件 `name` 选项使用 PascalCase。 | `opinionated`, `nuxt` |  |
| <span id="script-custom-event-name-casing"></span>[`script/custom-event-name-casing`](./reference/script-custom-event-name-casing.md) | 要求发出的自定义事件名称使用配置的大小写形式。 | `opinionated`, `nuxt` |  |
| <span id="script-define-emits-declaration"></span>[`script/define-emits-declaration`](./reference/script-define-emits-declaration.md) | 要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式。 | `opinionated`, `nuxt` |  |
| <span id="script-define-macros-order"></span>[`script/define-macros-order`](./reference/script-define-macros-order.md) | 要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序。 | `opinionated`, `nuxt` |  |
| <span id="script-define-props-declaration"></span>[`script/define-props-declaration`](./reference/script-define-props-declaration.md) | 要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式。 | `opinionated`, `nuxt` |  |
| <span id="script-no-boolean-default"></span>[`script/no-boolean-default`](./reference/script-no-boolean-default.md) | 禁止为 Boolean prop 设置默认值。 | `opinionated`, `nuxt` |  |
| <span id="script-no-deep-destructure-in-props"></span>[`script/no-deep-destructure-in-props`](./reference/script-no-deep-destructure-in-props.md) | 禁止在 defineProps 中进行深层嵌套解构。 | `opinionated`, `nuxt` |  |
| <span id="script-no-get-current-instance"></span>[`script/no-get-current-instance`](./reference/script-no-get-current-instance.md) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）。 | `opinionated` |  |
| <span id="script-no-options-api"></span>[`script/no-options-api`](./reference/script-no-options-api.md) | 禁止 Vapor 模式中的 Options API 模式。 | `opinionated` |  |
| <span id="script-no-unused-emit-declarations"></span>[`script/no-unused-emit-declarations`](./reference/script-no-unused-emit-declarations.md) | 标记已声明却从未发出的事件。 | `opinionated`, `nuxt` |  |
| <span id="script-no-with-defaults"></span>[`script/no-with-defaults`](./reference/script-no-with-defaults.md) | 不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）。 | `opinionated` |  |
| <span id="script-one-component-per-file"></span>[`script/one-component-per-file`](./reference/script-one-component-per-file.md) | 禁止在一个文件中定义多个组件。 | `opinionated`, `nuxt` |  |
| <span id="script-order-in-components"></span>[`script/order-in-components`](./reference/script-order-in-components.md) | 强制组件选项的属性遵循默认顺序。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-define-options"></span>[`script/prefer-define-options`](./reference/script-prefer-define-options.md) | 优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;。 | `opinionated`, `nuxt` |  |
| <span id="script-prefer-use-template-ref"></span>[`script/prefer-use-template-ref`](./reference/script-prefer-use-template-ref.md) | 模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+）。 | `opinionated`, `nuxt` |  |
| <span id="script-require-default-prop"></span>[`script/require-default-prop`](./reference/script-require-default-prop.md) | 要求每个可选的非 Boolean prop 具有默认值。 | `opinionated`, `nuxt` |  |
| <span id="script-require-explicit-slots"></span>[`script/require-explicit-slots`](./reference/script-require-explicit-slots.md) | 要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型。 | `opinionated`, `nuxt` |  |
| <span id="script-require-prop-types"></span>[`script/require-prop-types`](./reference/script-require-prop-types.md) | 要求每个 prop 声明类型。 | `opinionated` |  |
| <span id="script-require-typed-object-prop"></span>[`script/require-typed-object-prop`](./reference/script-require-typed-object-prop.md) | 要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型。 | `opinionated`, `nuxt` |  |
| <span id="type-await-thenable"></span>[`type/await-thenable`](./reference/type-await-thenable.md) | 禁止 await 非 thenable 的值。 | `nuxt`, `opinionated` |  |
| <span id="type-no-array-delete"></span>[`type/no-array-delete`](./reference/type-no-array-delete.md) | 禁止对类数组值使用 delete 删除元素。 | `nuxt`, `opinionated` |  |
| <span id="type-no-base-to-string"></span>[`type/no-base-to-string`](./reference/type-no-base-to-string.md) | 禁止把会回退到 Object.prototype.toString() 的值转换成字符串。 | `nuxt`, `opinionated` |  |
| <span id="type-no-confusing-void-expression"></span>[`type/no-confusing-void-expression`](./reference/type-no-confusing-void-expression.md) | 禁止在容易混淆的值位置使用 void 类型的表达式。 | `nuxt`, `opinionated` |  |
| <span id="type-no-deprecated"></span>[`type/no-deprecated`](./reference/type-no-deprecated.md) | 禁止使用已弃用的声明。 | `nuxt`, `opinionated` |  |
| <span id="type-no-duplicate-type-constituents"></span>[`type/no-duplicate-type-constituents`](./reference/type-no-duplicate-type-constituents.md) | 禁止联合类型或交叉类型中的重复成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-floating-promises"></span>[`type/no-floating-promises`](./reference/type-no-floating-promises.md) | 禁止悬空（未处理）的 Promise。 | `nuxt`, `opinionated` |  |
| <span id="type-no-for-in-array"></span>[`type/no-for-in-array`](./reference/type-no-for-in-array.md) | 禁止对类数组值使用 for-in 遍历。 | `nuxt`, `opinionated` |  |
| <span id="type-no-generated-empty-object-type"></span>[`type/no-generated-empty-object-type`](./reference/type-no-generated-empty-object-type.md) | 禁止由类型运算生成的空对象类型。 | `nuxt`, `opinionated` |  |
| <span id="type-no-implied-eval"></span>[`type/no-implied-eval`](./reference/type-no-implied-eval.md) | 禁止基于字符串的动态代码执行 API。 | `nuxt`, `opinionated` |  |
| <span id="type-no-meaningless-void-operator"></span>[`type/no-meaningless-void-operator`](./reference/type-no-meaningless-void-operator.md) | 禁止对已经是 void 的表达式使用 void 运算符。 | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-promises"></span>[`type/no-misused-promises`](./reference/type-no-misused-promises.md) | 禁止在并非为处理 Promise 而设计的位置使用 Promise。 | `nuxt`, `opinionated` |  |
| <span id="type-no-misused-spread"></span>[`type/no-misused-spread`](./reference/type-no-misused-spread.md) | 禁止把值展开到不兼容的位置。 | `nuxt`, `opinionated` |  |
| <span id="type-no-mixed-enums"></span>[`type/no-mixed-enums`](./reference/type-no-mixed-enums.md) | 禁止在枚举中混用字符串成员和数值成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-reactivity-loss"></span>[`type/no-reactivity-loss`](./reference/type-no-reactivity-loss.md) | 禁止赋值和调用中对响应式值取普通快照。 | `nuxt`, `opinionated` |  |
| <span id="type-no-redundant-type-constituents"></span>[`type/no-redundant-type-constituents`](./reference/type-no-redundant-type-constituents.md) | 禁止联合类型和交叉类型中不起作用或覆盖类型信息的成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-boolean-literal-compare"></span>[`type/no-unnecessary-boolean-literal-compare`](./reference/type-no-unnecessary-boolean-literal-compare.md) | 禁止与布尔字面量进行不必要的比较。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-condition"></span>[`type/no-unnecessary-condition`](./reference/type-no-unnecessary-condition.md) | 禁止始终为真、始终为假、never 或始终为 nullish 的条件。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-template-expression"></span>[`type/no-unnecessary-template-expression`](./reference/type-no-unnecessary-template-expression.md) | 禁止用不必要的模板字符串只包裹一个表达式。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-arguments"></span>[`type/no-unnecessary-type-arguments`](./reference/type-no-unnecessary-type-arguments.md) | 禁止与默认值相同的类型参数。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-assertion"></span>[`type/no-unnecessary-type-assertion`](./reference/type-no-unnecessary-type-assertion.md) | 禁止不改变表达式类型的类型断言。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unnecessary-type-conversion"></span>[`type/no-unnecessary-type-conversion`](./reference/type-no-unnecessary-type-conversion.md) | 禁止不改变表达式类型或值的转换。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-argument"></span>[`type/no-unsafe-argument`](./reference/type-no-unsafe-argument.md) | 禁止用 `any` 类型的值调用函数。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-assignment"></span>[`type/no-unsafe-assignment`](./reference/type-no-unsafe-assignment.md) | 禁止把 any 类型的值赋给更具体的目标。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-call"></span>[`type/no-unsafe-call`](./reference/type-no-unsafe-call.md) | 禁止调用类型为 any 的值。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-assignment"></span>[`type/no-unsafe-enum-assignment`](./reference/type-no-unsafe-enum-assignment.md) | 禁止把非枚举值赋给枚举。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-enum-comparison"></span>[`type/no-unsafe-enum-comparison`](./reference/type-no-unsafe-enum-comparison.md) | 禁止把枚举值与另一枚举域的值比较。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-member-access"></span>[`type/no-unsafe-member-access`](./reference/type-no-unsafe-member-access.md) | 禁止访问类型为 any 的值的成员。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-return"></span>[`type/no-unsafe-return`](./reference/type-no-unsafe-return.md) | 禁止从函数返回 any 类型的值。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-template-binding"></span>[`type/no-unsafe-template-binding`](./reference/type-no-unsafe-template-binding.md) | 禁止解析为不安全类型的模板绑定。 | `nuxt`, `opinionated` |  |
| <span id="type-no-unsafe-unary-minus"></span>[`type/no-unsafe-unary-minus`](./reference/type-no-unsafe-unary-minus.md) | 禁止对非 number 且非 bigint 的值使用一元负号。 | `nuxt`, `opinionated` |  |
| <span id="type-only-throw-error"></span>[`type/only-throw-error`](./reference/type-only-throw-error.md) | 要求抛出的值是 Error 一类的值。 | `nuxt`, `opinionated` |  |
| <span id="type-prefer-promise-reject-errors"></span>[`type/prefer-promise-reject-errors`](./reference/type-prefer-promise-reject-errors.md) | 要求 Promise 的拒绝原因是 Error 一类的值。 | `nuxt`, `opinionated` |  |
| <span id="type-prefer-reduce-type-parameter"></span>[`type/prefer-reduce-type-parameter`](./reference/type-prefer-reduce-type-parameter.md) | 推荐使用 Array#reduce 的类型参数，而不是对初始值做断言。 | `nuxt`, `opinionated` |  |
| <span id="type-prefer-return-this-type"></span>[`type/prefer-return-this-type`](./reference/type-prefer-return-this-type.md) | 要求在只返回 `this` 类型时使用 `this`。 | `nuxt`, `opinionated` |  |
| <span id="type-related-getter-setter-pairs"></span>[`type/related-getter-setter-pairs`](./reference/type-related-getter-setter-pairs.md) | 要求成对的 getter 和 setter 使用兼容的类型。 | `nuxt`, `opinionated` |  |
| <span id="type-require-await"></span>[`type/require-await`](./reference/type-require-await.md) | 禁止不包含 await 表达式的 async 函数。 | `nuxt`, `opinionated` |  |
| <span id="type-restrict-plus-operands"></span>[`type/restrict-plus-operands`](./reference/type-restrict-plus-operands.md) | 要求加号的操作数明确兼容。 | `nuxt`, `opinionated` |  |
| <span id="type-restrict-template-expressions"></span>[`type/restrict-template-expressions`](./reference/type-restrict-template-expressions.md) | 要求模板字符串中的表达式与字符串兼容。 | `nuxt`, `opinionated` |  |
| <span id="type-return-await"></span>[`type/return-await`](./reference/type-return-await.md) | 要求根据上下文和配置一致地使用 `return await`。 | `nuxt`, `opinionated` |  |
| <span id="type-unbound-method"></span>[`type/unbound-method`](./reference/type-unbound-method.md) | 禁止在没有接收者（`this`）的情况下引用方法。 | `nuxt`, `opinionated` |  |
| <span id="type-use-unknown-in-catch-callback-variable"></span>[`type/use-unknown-in-catch-callback-variable`](./reference/type-use-unknown-in-catch-callback-variable.md) | 要求 Promise 的 catch 回调参数使用显式的 unknown 注解。 | `nuxt`, `opinionated` |  |

## 按需启用 (16)

不属于任何预设。在传给 `withVue` 的 `linter.rules` 中按名称启用。

| 规则 | 检查内容 | 自动修复 |
| --- | --- | --- |
| <span id="script-consistent-props-emits-key-quotes"></span>[`script/consistent-props-emits-key-quotes`](./reference/script-consistent-props-emits-key-quotes.md) | 统一 props 和 emits 声明中带引号的键。 |  |
| <span id="script-no-next-tick"></span>[`script/no-next-tick`](./reference/script-no-next-tick.md) | 禁止面向 Vapor 的组件使用 nextTick()。 |  |
| <span id="script-no-reactive-destructure"></span>[`script/no-reactive-destructure`](./reference/script-no-reactive-destructure.md) | 禁止导致响应性丢失的响应式对象解构。 |  |
| <span id="script-no-restricted-globals"></span>[`script/no-restricted-globals`](./reference/script-no-restricted-globals.md) | 禁止直接引用必须通过类型化包装器访问的运行时环境全局变量。 |  |
| <span id="script-no-restricted-members"></span>[`script/no-restricted-members`](./reference/script-no-restricted-members.md) | 禁止项目配置的 object.property 成员访问。 |  |
| <span id="script-no-top-level-ref-in-script"></span>[`script/no-top-level-ref-in-script`](./reference/script-no-top-level-ref-in-script.md) | 禁止顶层 ref/reactive，以防止跨请求状态污染。 |  |
| <span id="script-prefer-computed"></span>[`script/prefer-computed`](./reference/script-prefer-computed.md) | 响应式派生状态优先使用 computed()。 |  |
| <span id="script-prefer-define-emits-binding"></span>[`script/prefer-define-emits-binding`](./reference/script-prefer-define-emits-binding.md) | 优先使用从 defineEmits 捕获的函数而不是模板 $emit。 | ✓ |
| <span id="script-prefer-ref-over-reactive"></span>[`script/prefer-ref-over-reactive`](./reference/script-prefer-ref-over-reactive.md) | 建议使用 ref() 而不是 reactive() 管理状态。 |  |
| <span id="script-prefer-use-attrs"></span>[`script/prefer-use-attrs`](./reference/script-prefer-use-attrs.md) | 建议使用 useAttrs() 而不是 context.attrs。 |  |
| <span id="script-prefer-use-id"></span>[`script/prefer-use-id`](./reference/script-prefer-use-id.md) | 建议使用 useId() 生成唯一 ID（Vue 3.5+）。 |  |
| <span id="script-prefer-use-slots"></span>[`script/prefer-use-slots`](./reference/script-prefer-use-slots.md) | 建议使用 useSlots() 而不是 context.slots。 |  |
| <span id="script-require-function-return-type"></span>[`script/require-function-return-type`](./reference/script-require-function-return-type.md) | 要求函数具有返回类型注解。 |  |
| <span id="script-require-symbol-provide"></span>[`script/require-symbol-provide`](./reference/script-require-symbol-provide.md) | 建议使用 Symbol 作为 provide/inject 的注入键。 |  |
| <span id="script-require-typed-ref"></span>[`script/require-typed-ref`](./reference/script-require-typed-ref.md) | 要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数。 |  |
| <span id="type-strict-boolean-expressions"></span>[`type/strict-boolean-expressions`](./reference/type-strict-boolean-expressions.md) | 要求脚本和模板条件使用安全的布尔表达式。 |  |

[规则概览](../rules/index.md) · [规则选项](/rules/options.md) · [从 ESLint 迁移](/rules/migration.md)
