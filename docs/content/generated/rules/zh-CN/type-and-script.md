---
title: "类型与脚本规则"
---

# 类型与脚本规则

本页集中列出所有相关规则的用途、配置、错误示例和正确示例，无需跳转页面查看对照。高亮行标示修改；复制代码时保留完整源代码。每条规则注明适用范围和当前支持限制；规范示例并不保证当前实现会产生诊断。

<span id="类型与文字规则"></span>
<span id="检查器配置"></span>

| 规则 | 示例 | 用途 |
| --- | --- | --- |
| [`script/component-options-name-casing`](#script-component-options-name-casing) | [错误示例](#script-component-options-name-casing-bad) · [正确示例](#script-component-options-name-casing-good) | 要求组件 `name` 选项使用 PascalCase |
| [`script/custom-event-name-casing`](#script-custom-event-name-casing) | [错误示例](#script-custom-event-name-casing-bad) · [正确示例](#script-custom-event-name-casing-good) | 要求发出的自定义事件名称使用 camelCase |
| [`script/define-emits-declaration`](#script-define-emits-declaration) | [错误示例](#script-define-emits-declaration-bad) · [正确示例](#script-define-emits-declaration-good) | 要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式 |
| [`script/define-macros-order`](#script-define-macros-order) | [错误示例](#script-define-macros-order-bad) · [正确示例](#script-define-macros-order-good) | 要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序 |
| [`script/define-props-declaration`](#script-define-props-declaration) | [错误示例](#script-define-props-declaration-bad) · [正确示例](#script-define-props-declaration-good) | 要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式 |
| [`script/define-props-destructuring`](#script-define-props-destructuring) | [错误示例](#script-define-props-destructuring-bad) · [正确示例](#script-define-props-destructuring-good) | 要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致 |
| [`script/no-arrow-functions-in-watch`](#script-no-arrow-functions-in-watch) | [错误示例](#script-no-arrow-functions-in-watch-bad) · [正确示例](#script-no-arrow-functions-in-watch-good) | 禁止将箭头函数用作 Options API watch 处理器 |
| [`script/no-async-in-computed`](#script-no-async-in-computed) | [错误示例](#script-no-async-in-computed-bad) · [正确示例](#script-no-async-in-computed-good) | 禁止计算属性中的异步函数 |
| [`script/no-boolean-default`](#script-no-boolean-default) | [错误示例](#script-no-boolean-default-bad) · [正确示例](#script-no-boolean-default-good) | 禁止为 Boolean prop 设置默认值 |
| [`script/no-deep-destructure-in-props`](#script-no-deep-destructure-in-props) | [错误示例](#script-no-deep-destructure-in-props-bad) · [正确示例](#script-no-deep-destructure-in-props-good) | 禁止在 defineProps 中进行深层嵌套解构 |
| [`script/no-deprecated-data-object-declaration`](#script-no-deprecated-data-object-declaration) | [错误示例](#script-no-deprecated-data-object-declaration-bad) · [正确示例](#script-no-deprecated-data-object-declaration-good) | 禁止将对象字面量作为组件 data 选项（Vue 3 要求函数） |
| [`script/no-deprecated-destroyed-lifecycle`](#script-no-deprecated-destroyed-lifecycle) | [错误示例](#script-no-deprecated-destroyed-lifecycle-bad) · [正确示例](#script-no-deprecated-destroyed-lifecycle-good) | 禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子 |
| [`script/no-deprecated-dollar-listeners-api`](#script-no-deprecated-dollar-listeners-api) | [错误示例](#script-no-deprecated-dollar-listeners-api-bad) · [正确示例](#script-no-deprecated-dollar-listeners-api-good) | 禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs） |
| [`script/no-deprecated-dollar-scopedslots-api`](#script-no-deprecated-dollar-scopedslots-api) | [错误示例](#script-no-deprecated-dollar-scopedslots-api-bad) · [正确示例](#script-no-deprecated-dollar-scopedslots-api-good) | 禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots） |
| [`script/no-deprecated-events-api`](#script-no-deprecated-events-api) | [错误示例](#script-no-deprecated-events-api-bad) · [正确示例](#script-no-deprecated-events-api-good) | 禁止已移除的 Vue 2 事件 API（$on / $off / $once） |
| [`script/no-deprecated-props-default-this`](#script-no-deprecated-props-default-this) | [错误示例](#script-no-deprecated-props-default-this-bad) · [正确示例](#script-no-deprecated-props-default-this-good) | 禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除） |
| [`script/no-dupe-keys`](#script-no-dupe-keys) | [错误示例](#script-no-dupe-keys-bad) · [正确示例](#script-no-dupe-keys-good) | 禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键 |
| [`script/no-duplicate-attr-inheritance`](#script-no-duplicate-attr-inheritance) | [错误示例](#script-no-duplicate-attr-inheritance-bad) · [正确示例](#script-no-duplicate-attr-inheritance-good) | 标记重复应用透传属性的组件 |
| [`script/no-export-in-script-setup`](#script-no-export-in-script-setup) | [错误示例](#script-no-export-in-script-setup-bad) · [正确示例](#script-no-export-in-script-setup-good) | 禁止 &lt;script setup&gt; 内的 export 语句 |
| [`script/no-get-current-instance`](#script-no-get-current-instance) | [错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good) | 禁止在 Vapor 模式使用 getCurrentInstance()（返回 null） |
| [`script/no-import-compiler-macros`](#script-no-import-compiler-macros) | [错误示例](#script-no-import-compiler-macros-bad) · [正确示例](#script-no-import-compiler-macros-good) | 禁止导入自动可用的 Vue 编译器宏 |
| [`script/no-internal-imports`](#script-no-internal-imports) | [错误示例](#script-no-internal-imports-bad) · [正确示例](#script-no-internal-imports-good) | 禁止从 Vue 内部模块导入 |
| [`script/no-multiple-slot-args`](#script-no-multiple-slot-args) | [错误示例](#script-no-multiple-slot-args-bad) · [正确示例](#script-no-multiple-slot-args-good) | 禁止向作用域插槽函数调用传入多个参数 |
| [`script/no-next-tick`](#script-no-next-tick) | [错误示例](#script-no-next-tick-bad) · [正确示例](#script-no-next-tick-good) | 禁止面向 Vapor 的组件使用 nextTick() |
| [`script/no-options-api`](#script-no-options-api) | [错误示例](#script-no-options-api-bad) · [正确示例](#script-no-options-api-good) | 禁止 Vapor 模式中的 Options API 模式 |
| [`script/no-potential-component-option-typo`](#script-no-potential-component-option-typo) | [错误示例](#script-no-potential-component-option-typo-bad) · [正确示例](#script-no-potential-component-option-typo-good) | 标记 Options API 组件选项名称中的疑似拼写错误 |
| [`script/no-reactive-destructure`](#script-no-reactive-destructure) | [错误示例](#script-no-reactive-destructure-bad) · [正确示例](#script-no-reactive-destructure-good) | 禁止导致响应性丢失的响应式对象解构 |
| [`script/no-ref-as-operand`](#script-no-ref-as-operand) | [错误示例](#script-no-ref-as-operand-bad) · [正确示例](#script-no-ref-as-operand-good) | 要求将 ref 绑定变量用作操作数时通过 `.value` 访问 |
| [`script/no-required-prop-with-default`](#script-no-required-prop-with-default) | [错误示例](#script-no-required-prop-with-default-bad) · [正确示例](#script-no-required-prop-with-default-good) | 禁止 prop 同时具有 required: true 和默认值 |
| [`script/no-reserved-identifiers`](#script-no-reserved-identifiers) | [错误示例](#script-no-reserved-identifiers-bad) · [正确示例](#script-no-reserved-identifiers-good) | 禁止使用 Vue 编译器保留标识符 |
| [`script/no-reserved-keys`](#script-no-reserved-keys) | [错误示例](#script-no-reserved-keys-bad) · [正确示例](#script-no-reserved-keys-good) | 禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键 |
| [`script/no-reserved-props`](#script-no-reserved-props) | [错误示例](#script-no-reserved-props-bad) · [正确示例](#script-no-reserved-props-good) | 禁止在组件 props 声明中使用保留名称 |
| [`script/no-restricted-globals`](#script-no-restricted-globals) | [错误示例](#script-no-restricted-globals-bad) · [正确示例](#script-no-restricted-globals-good) | 禁止直接引用必须通过类型化包装器访问的运行时环境全局变量 |
| [`script/no-restricted-members`](#script-no-restricted-members) | [错误示例](#script-no-restricted-members-bad) · [正确示例](#script-no-restricted-members-good) | 禁止项目配置的 object.property 成员访问 |
| [`script/no-side-effects-in-computed-properties`](#script-no-side-effects-in-computed-properties) | [错误示例](#script-no-side-effects-in-computed-properties-bad) · [正确示例](#script-no-side-effects-in-computed-properties-good) | 禁止 Options API 计算 getter 中的副作用 |
| [`script/no-top-level-ref-in-script`](#script-no-top-level-ref-in-script) | [错误示例](#script-no-top-level-ref-in-script-bad) · [正确示例](#script-no-top-level-ref-in-script-good) | 禁止顶层 ref/reactive，以防止跨请求状态污染 |
| [`script/no-unstable-nested-components`](#script-no-unstable-nested-components) | [错误示例](#script-no-unstable-nested-components-bad) · [正确示例](#script-no-unstable-nested-components-good) | 禁止在 setup 或 render 函数内定义组件 |
| [`script/no-unused-emit-declarations`](#script-no-unused-emit-declarations) | [错误示例](#script-no-unused-emit-declarations-bad) · [正确示例](#script-no-unused-emit-declarations-good) | 标记已声明却从未发出的事件 |
| [`script/no-use-computed-property-like-method`](#script-no-use-computed-property-like-method) | [错误示例](#script-no-use-computed-property-like-method-bad) · [正确示例](#script-no-use-computed-property-like-method-good) | 禁止像方法一样调用 Options API 计算属性 |
| [`script/no-with-defaults`](#script-no-with-defaults) | [错误示例](#script-no-with-defaults-bad) · [正确示例](#script-no-with-defaults-good) | 不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+） |
| [`script/prefer-computed`](#script-prefer-computed) | [错误示例](#script-prefer-computed-bad) · [正确示例](#script-prefer-computed-good) | 响应式派生状态优先使用 computed() |
| [`script/prefer-define-options`](#script-prefer-define-options) | [错误示例](#script-prefer-define-options-bad) · [正确示例](#script-prefer-define-options-good) | 优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt; |
| [`script/prefer-import-from-vue`](#script-prefer-import-from-vue) | [错误示例](#script-prefer-import-from-vue-bad) · [正确示例](#script-prefer-import-from-vue-good) | 优先从 'vue' 导入，而不是内部包 |
| [`script/prefer-ref-over-reactive`](#script-prefer-ref-over-reactive) | [错误示例](#script-prefer-ref-over-reactive-bad) · [正确示例](#script-prefer-ref-over-reactive-good) | 建议使用 ref() 而不是 reactive() 管理状态 |
| [`script/prefer-use-attrs`](#script-prefer-use-attrs) | [错误示例](#script-prefer-use-attrs-bad) · [正确示例](#script-prefer-use-attrs-good) | 建议使用 useAttrs() 而不是 context.attrs |
| [`script/prefer-use-id`](#script-prefer-use-id) | [错误示例](#script-prefer-use-id-bad) · [正确示例](#script-prefer-use-id-good) | 建议使用 useId() 生成唯一 ID（Vue 3.5+） |
| [`script/prefer-use-slots`](#script-prefer-use-slots) | [错误示例](#script-prefer-use-slots-bad) · [正确示例](#script-prefer-use-slots-good) | 建议使用 useSlots() 而不是 context.slots |
| [`script/prefer-use-template-ref`](#script-prefer-use-template-ref) | [错误示例](#script-prefer-use-template-ref-bad) · [正确示例](#script-prefer-use-template-ref-good) | 模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+） |
| [`script/require-default-prop`](#script-require-default-prop) | [错误示例](#script-require-default-prop-bad) · [正确示例](#script-require-default-prop-good) | 要求每个可选的非 Boolean prop 具有默认值 |
| [`script/require-explicit-emits`](#script-require-explicit-emits) | [错误示例](#script-require-explicit-emits-bad) · [正确示例](#script-require-explicit-emits-good) | 要求发出的事件在 defineEmits 或 emits 选项中声明 |
| [`script/require-explicit-slots`](#script-require-explicit-slots) | [错误示例](#script-require-explicit-slots-bad) · [正确示例](#script-require-explicit-slots-good) | 要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型 |
| [`script/require-function-return-type`](#script-require-function-return-type) | [错误示例](#script-require-function-return-type-bad) · [正确示例](#script-require-function-return-type-good) | 要求函数具有返回类型注解 |
| [`script/require-prop-type-constructor`](#script-require-prop-type-constructor) | [错误示例](#script-require-prop-type-constructor-bad) · [正确示例](#script-require-prop-type-constructor-good) | 要求 prop 的 `type` 值是构造器，而不是字符串字面量 |
| [`script/require-prop-types`](#script-require-prop-types) | [错误示例](#script-require-prop-types-bad) · [正确示例](#script-require-prop-types-good) | 要求每个 prop 声明类型 |
| [`script/require-symbol-provide`](#script-require-symbol-provide) | [错误示例](#script-require-symbol-provide-bad) · [正确示例](#script-require-symbol-provide-good) | 建议使用 Symbol 作为 provide/inject 的注入键 |
| [`script/require-typed-object-prop`](#script-require-typed-object-prop) | [错误示例](#script-require-typed-object-prop-bad) · [正确示例](#script-require-typed-object-prop-good) | 要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型 |
| [`script/require-typed-ref`](#script-require-typed-ref) | [错误示例](#script-require-typed-ref-bad) · [正确示例](#script-require-typed-ref-good) | 要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数 |
| [`script/require-valid-default-prop`](#script-require-valid-default-prop) | [错误示例](#script-require-valid-default-prop-bad) · [正确示例](#script-require-valid-default-prop-good) | 要求 prop 默认值符合声明类型 |
| [`script/return-in-computed-property`](#script-return-in-computed-property) | [错误示例](#script-return-in-computed-property-bad) · [正确示例](#script-return-in-computed-property-good) | 要求每个计算 getter 返回值 |
| [`script/return-in-emits-validator`](#script-return-in-emits-validator) | [错误示例](#script-return-in-emits-validator-bad) · [正确示例](#script-return-in-emits-validator-good) | 要求每个 Options API emits 校验函数返回值 |
| [`script/valid-define-emits`](#script-valid-define-emits) | [错误示例](#script-valid-define-emits-bad) · [正确示例](#script-valid-define-emits-good) | 要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次） |
| [`script/valid-define-options`](#script-valid-define-options) | [错误示例](#script-valid-define-options-bad) · [正确示例](#script-valid-define-options-good) | 要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots） |
| [`script/valid-define-props`](#script-valid-define-props) | [错误示例](#script-valid-define-props-bad) · [正确示例](#script-valid-define-props-good) | 要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量） |
| [`script/valid-next-tick`](#script-valid-next-tick) | [错误示例](#script-valid-next-tick-bad) · [正确示例](#script-valid-next-tick-good) | 要求 nextTick() 调用结果被等待、链式处理，或提供回调 |
| [`type/no-floating-promises`](#type-no-floating-promises) | [错误示例](#type-no-floating-promises-bad) · [正确示例](#type-no-floating-promises-good) | 禁止悬空（未处理）的 Promise |
| [`type/no-reactivity-loss`](#type-no-reactivity-loss) | [错误示例](#type-no-reactivity-loss-bad) · [正确示例](#type-no-reactivity-loss-good) | 禁止赋值和调用中对响应式值取普通快照 |
| [`type/no-unsafe-template-binding`](#type-no-unsafe-template-binding) | [错误示例](#type-no-unsafe-template-binding-bad) · [正确示例](#type-no-unsafe-template-binding-good) | 禁止解析为不安全类型的模板绑定 |
| [`type/require-typed-emits`](#type-require-typed-emits) | [错误示例](#type-require-typed-emits-bad) · [正确示例](#type-require-typed-emits-good) | 要求 defineEmits 具有类型定义 |
| [`type/require-typed-props`](#type-require-typed-props) | [错误示例](#type-require-typed-props-bad) · [正确示例](#type-require-typed-props-good) | 要求 defineProps 具有类型定义 |
| [`type/strict-boolean-expressions`](#type-strict-boolean-expressions) | [错误示例](#type-strict-boolean-expressions-bad) · [正确示例](#type-strict-boolean-expressions-good) | 要求脚本和模板条件使用安全的布尔表达式 |

[全部规则](./all.md) · [规则选项](/rules/options.md) · [ESLint 迁移对应表](/rules/migration.md) · [项目检查](./cross-file.md) · [组件间属性传递](/rules/project/vue-cross-file-attrs-fallthrough.md)

### `script/component-options-name-casing`

要求组件 `name` 选项使用 PascalCase

[错误示例](#script-component-options-name-casing-bad) · [正确示例](#script-component-options-name-casing-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/component-options-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-component-options-name-casing-bad"></span>

**错误示例**

组件选项 `name: 'my-component'` 使用 kebab-case，但此规则要求名称字面量采用 PascalCase。

```vue annotate="remove:3"
<script lang="ts">
export default {
name: 'my-component' // kebab-case
}
</script>
```

<span id="script-component-options-name-casing-good"></span>

**正确示例**

`MyComponent` 以大写字母开头，且仅包含字母和数字，满足名称检查。

```vue annotate="add:3"
<script lang="ts">
export default {
name: 'MyComponent'
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/component_options_name_casing.rs#L44) · [全部规则](all.md)

### `script/custom-event-name-casing`

要求发出的自定义事件名称使用 camelCase

[错误示例](#script-custom-event-name-casing-bad) · [正确示例](#script-custom-event-name-casing-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/custom-event-name-casing": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-custom-event-name-casing-bad"></span>

**错误示例**

发出的字符串 `my-event` 含有连字符，违反默认的 camelCase 事件命名策略。

```vue annotate="remove:2,3"
<script setup lang="ts">
const emit = defineEmits(['my-event'])
emit('my-event')         // kebab-case → report
</script>
```

<span id="script-custom-event-name-casing-good"></span>

**正确示例**

声明和调用都使用 `myEvent`，既保持事件名称与发出操作一致，也满足默认大小写策略。配置为 kebab-case 时要求不同。

```vue annotate="add:2,3"
<script setup lang="ts">
const emit = defineEmits(['myEvent'])
emit('myEvent')
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/custom_event_name_casing.rs#L61) · [全部规则](all.md)

### `script/define-emits-declaration`

要求使用类型形式 defineEmits&lt;{}&gt;()，而不是运行时或数组形式

[错误示例](#script-define-emits-declaration-bad) · [正确示例](#script-define-emits-declaration-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-emits-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-emits-declaration-bad"></span>

**错误示例**

`defineEmits(["change"])` 使用运行时数组声明；此风格规则偏好类型声明。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits(["change"]);
emit("change", 1);
</script>
```

<span id="script-define-emits-declaration-good"></span>

**正确示例**

`defineEmits<{ change: [id: number] }>()` 将事件声明移到类型参数中，并显式描述 `emit("change", 1)` 使用的数字载荷。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits<{ change: [id: number] }>();
emit("change", 1);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_emits_declaration.rs#L39) · [全部规则](all.md)

### `script/define-macros-order`

要求 &lt;script setup&gt; 中的 Vue 编译器宏保持一致的顺序

[错误示例](#script-define-macros-order-bad) · [正确示例](#script-define-macros-order-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-macros-order": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-macros-order-bad"></span>

**错误示例**

`defineProps` 位于 `defineModel` 之前，但标准宏顺序中 `defineModel` 更靠前。

```vue annotate="remove:2,3"
<script setup lang="ts">
// defineProps before defineModel (out of canonical order)
const props = defineProps<{ count: number }>()
const model = defineModel<string>()
</script>
```

<span id="script-define-macros-order-good"></span>

**正确示例**

声明严格按照 `defineOptions`、`defineModel`、`defineProps`、`defineEmits`、`defineSlots` 排列，再写无关的运行时语句。

```vue annotate="add:2,4,5,6"
<script setup lang="ts">
defineOptions({ name: 'MyComponent' })
const model = defineModel<string>()
const props = defineProps<{ count: number }>()
const emit = defineEmits<{ change: [value: string] }>()
defineSlots<{ default(props: {}): any }>()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_macros_order.rs#L46) · [全部规则](all.md)

### `script/define-props-declaration`

要求使用类型形式 defineProps&lt;{ ... }&gt;()，而不是运行时或对象形式

[错误示例](#script-define-props-declaration-bad) · [正确示例](#script-define-props-declaration-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-declaration": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-declaration-bad"></span>

**错误示例**

`defineProps({ title: String })` 提供运行时对象，与此规则偏好的类型 props 声明冲突。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ title: String });
console.log(props.title);
</script>
```

<span id="script-define-props-declaration-good"></span>

**正确示例**

`defineProps<{ title: string }>()` 在类型参数中声明 `title`，保留 `props.title` 访问，无需运行时声明参数。

```vue annotate="add:2"
<script setup lang="ts">
const props = defineProps<{ title: string }>();
console.log(props.title);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_declaration.rs#L40) · [全部规则](all.md)

### `script/define-props-destructuring`

要求 &lt;script setup&gt; 中的 defineProps 解构风格保持一致

[错误示例](#script-define-props-destructuring-bad) · [正确示例](#script-define-props-destructuring-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/define-props-destructuring": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-define-props-destructuring-bad"></span>

**错误示例**

`defineProps` 被赋给单一的 `props` 绑定，而非解构，不符合默认的解构偏好。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ foo: string }>()
</script>
```

<span id="script-define-props-destructuring-good"></span>

**正确示例**

对象模式直接绑定 `foo` 和 `bar`，并为可选的 `bar` 提供默认值。这依赖 Vue 3.5+ 的响应式 props 解构；可配置的 `never` 模式偏好相反形式。

```vue annotate="add:2"
<script setup lang="ts">
const { foo, bar = 'default' } = defineProps<{ foo: string; bar?: string }>()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/define_props_destructuring.rs#L29) · [全部规则](all.md)

### `script/no-arrow-functions-in-watch`

禁止将箭头函数用作 Options API watch 处理器

[错误示例](#script-no-arrow-functions-in-watch-bad) · [正确示例](#script-no-arrow-functions-in-watch-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-arrow-functions-in-watch": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-arrow-functions-in-watch-bad"></span>

**错误示例**

Options API 侦听器 `value` 和嵌套的 `other.handler` 都是箭头函数。箭头函数捕获外层 `this`，不会接收组件实例。

```vue annotate="remove:4,5,9"
<script lang="ts">
export default {
watch: {
// `this` is not the component instance inside an arrow function.
value: () => {
this.doSomething()
},
other: {
handler: () => {}
}
}
}
</script>
```

<span id="script-no-arrow-functions-in-watch-good"></span>

**正确示例**

两个处理器改为普通方法，使 Vue 能将 `this` 绑定到组件。`deep: true` 侦听选项仍与对象形式兼容。

```vue annotate="add:4,8,9"
<script lang="ts">
export default {
watch: {
value(newValue, oldValue) {
this.doSomething()
},
other: {
handler(newValue) {},
deep: true
}
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_arrow_functions_in_watch.rs#L59) · [全部规则](all.md)

### `script/no-async-in-computed`

禁止计算属性中的异步函数

[错误示例](#script-no-async-in-computed-bad) · [正确示例](#script-no-async-in-computed-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-async-in-computed": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-async-in-computed-bad"></span>

**错误示例**

`computed` getter 使用 `async`，因此 fetch 产生 Promise，而不是同步派生的计算值。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
import { computed } from "vue";
const data = computed(async () => {
  const response = await fetch("/api/data");
  return response.json();
});
</script>
```

<span id="script-no-async-in-computed-good"></span>

**正确示例**

异步 fetch 移到 `watch`，将结果存入 `data.value`。清理逻辑中止旧请求，并阻止失效回调写入过期结果；不再存在异步计算 getter。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
import { ref, watch } from "vue";
const query = ref("");
const data = ref<unknown>(null);
watch(query, async (value, _oldValue, onCleanup) => {
  const controller = new AbortController();
  let active = true;
  onCleanup(() => { active = false; controller.abort(); });
  const response = await fetch(`/api/data?q=${encodeURIComponent(value)}`, { signal: controller.signal });
  const next: unknown = await response.json();
  if (active) data.value = next;
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_async_in_computed.rs#L46) · [全部规则](all.md)

### `script/no-boolean-default`

禁止为 Boolean prop 设置默认值

[错误示例](#script-no-boolean-default-bad) · [正确示例](#script-no-boolean-default-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-boolean-default": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-boolean-default-bad"></span>

**错误示例**

`disabled` 和 `checked` 都为唯一构造器为 `Boolean` 的 prop 声明了 `default`；规则连显式的 `false` 默认值也会拒绝。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// Boolean props already default to false; an explicit default is confusing.
disabled: { type: Boolean, default: true },
checked: { type: Boolean, default: false }
}
}
</script>
```

<span id="script-no-boolean-default-good"></span>

**正确示例**

仅 Boolean 的 props 省略 `default`，使用 Vue 隐式的 false 值。`[Boolean, String]` 联合类型和 Number prop 说明此检查仅限唯一的 `Boolean` 构造器。

```vue annotate="add:4,5,6,7,8,9,10"
<script lang="ts">
export default {
props: {
// No explicit default: defaults to false.
disabled: { type: Boolean },
disabled2: Boolean,
// Union type may legitimately need a default.
value: { type: [Boolean, String], default: '' },
// Non-Boolean prop.
count: { type: Number, default: 0 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_boolean_default.rs#L54) · [全部规则](all.md)

### `script/no-deep-destructure-in-props`

禁止在 defineProps 中进行深层嵌套解构

[错误示例](#script-no-deep-destructure-in-props-bad) · [正确示例](#script-no-deep-destructure-in-props-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deep-destructure-in-props": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deep-destructure-in-props-bad"></span>

**错误示例**

绑定模式进入 `user` 解构 `name`，超过默认的浅层 props 解构深度。

```vue annotate="remove:2"
<script setup lang="ts">
const { user: { name } } = defineProps<{ user: { name: string } }>();
</script>
```

<span id="script-no-deep-destructure-in-props-good"></span>

**正确示例**

props 对象保持完整，由计算 getter 读取 `props.user.name`。嵌套访问保持显式，不使用深层嵌套绑定模式。

```vue annotate="add:2,3,4"
<script setup lang="ts">
import { computed } from "vue";
const props = defineProps<{ user: { name: string } }>();
const userName = computed(() => props.user.name);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deep_destructure_in_props.rs#L37) · [全部规则](all.md)

### `script/no-deprecated-data-object-declaration`

禁止将对象字面量作为组件 data 选项（Vue 3 要求函数）

[错误示例](#script-no-deprecated-data-object-declaration-bad) · [正确示例](#script-no-deprecated-data-object-declaration-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-data-object-declaration": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-data-object-declaration-bad"></span>

**错误示例**

Options API 的 `data` 选项是对象字面量，这是 Vue 3 不再接受的 Vue 2 形式。

```vue annotate="remove:3,4,5"
<script lang="ts">
export default {
// `data` must be a function in Vue 3, not an object literal.
data: {
count: 0
}
}
</script>
```

<span id="script-no-deprecated-data-object-declaration-good"></span>

**正确示例**

`data()` 返回新的 `{ count: 0 }` 对象，提供 Vue 3 所需的函数式 data 声明。

```vue annotate="add:3,4"
<script lang="ts">
export default {
data() {
return { count: 0 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_data_object_declaration.rs#L48) · [全部规则](all.md)

### `script/no-deprecated-destroyed-lifecycle`

禁止已弃用的 destroyed 和 beforeDestroy 生命周期钩子

[错误示例](#script-no-deprecated-destroyed-lifecycle-bad) · [正确示例](#script-no-deprecated-destroyed-lifecycle-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-destroyed-lifecycle": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-destroyed-lifecycle-bad"></span>

**错误示例**

定时器清理使用了已移除的 Vue 2 生命周期选项 `beforeDestroy`。

```vue annotate="remove:2"
<script lang="ts">
export default { beforeDestroy() { clearTimeout(this.timer); } };
</script>
```

<span id="script-no-deprecated-destroyed-lifecycle-good"></span>

**正确示例**

将钩子重命名为 `beforeUnmount`，在 Vue 3 生命周期名称下保留清理函数体。

```vue annotate="add:2"
<script lang="ts">
export default { beforeUnmount() { clearTimeout(this.timer); } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_destroyed_lifecycle.rs#L17) · [全部规则](all.md)

### `script/no-deprecated-dollar-listeners-api`

禁止 Vue 3 已移除的 $listeners 实例属性（已合并到 $attrs）

[错误示例](#script-no-deprecated-dollar-listeners-api-bad) · [正确示例](#script-no-deprecated-dollar-listeners-api-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-listeners-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-listeners-api-bad"></span>

**错误示例**

成员读取和独立参数引用都使用 `$listeners`，Vue 3 在将监听器合并到属性后移除了它。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const handlers = this.$listeners
const forwarded = ctx.$listeners
emit('input', $listeners)
</script>
```

<span id="script-no-deprecated-dollar-listeners-api-good"></span>

**正确示例**

读取改为 `this.$attrs` 和 setup 上下文的 `ctx.attrs`，替代已移除的监听器接口；示例中的接收对象必须存在于周围组件上下文。

```vue annotate="add:2,3"
<script setup lang="ts">
const handlers = this.$attrs
const forwarded = ctx.attrs
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_listeners_api.rs#L40) · [全部规则](all.md)

### `script/no-deprecated-dollar-scopedslots-api`

禁止 Vue 3 已移除的 $scopedSlots 实例属性（使用 $slots）

[错误示例](#script-no-deprecated-dollar-scopedslots-api-bad) · [正确示例](#script-no-deprecated-dollar-scopedslots-api-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-dollar-scopedslots-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-dollar-scopedslots-api-bad"></span>

**错误示例**

`this.$scopedSlots`、`ctx.$scopedSlots` 和独立的 `$scopedSlots` 引用使用了 Vue 3 已移除的 Vue 2 作用域插槽 API。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const header = this.$scopedSlots.header
const footer = ctx.$scopedSlots.footer
render($scopedSlots.default)
</script>
```

<span id="script-no-deprecated-dollar-scopedslots-api-good"></span>

**正确示例**

将 `$scopedSlots` 替换为 `$slots`，使用统一插槽接口。示例移除弃用写法，并未为接收对象建立 setup 上下文。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const header = this.$slots.header
const footer = ctx.$slots.footer
render($slots.default)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_dollar_scopedslots_api.rs#L44) · [全部规则](all.md)

### `script/no-deprecated-events-api`

禁止已移除的 Vue 2 事件 API（$on / $off / $once）

[错误示例](#script-no-deprecated-events-api-bad) · [正确示例](#script-no-deprecated-events-api-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-events-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-events-api-bad"></span>

**错误示例**

`$on`、`$once` 和 `$off` 调用使用了 Vue 3 已移除的实例事件总线方法。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
this.$on('event', handler)
this.$once('event', handler)
this.$off('event', handler)
emitter.$off('event')
</script>
```

<span id="script-no-deprecated-events-api-good"></span>

**正确示例**

`$emit` 仍然有效，事件总线订阅则移到外部发射器的 `on` 方法。修复区分面向父组件的事件发出与外部事件总线。

```vue annotate="add:2,3,4,5,6,7,8"
<script setup lang="ts">
// $emit is still valid in Vue 3
this.$emit('event', payload)

// Use an external emitter instead
import mitt from 'mitt'
const emitter = mitt()
emitter.on('event', handler)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_events_api.rs#L42) · [全部规则](all.md)

### `script/no-deprecated-props-default-this`

禁止在 prop 默认值或校验函数中使用 `this`（Vue 3 已移除）

[错误示例](#script-no-deprecated-props-default-this-bad) · [正确示例](#script-no-deprecated-props-default-this-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-deprecated-props-default-this": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-deprecated-props-default-this-bad"></span>

**错误示例**

prop 默认值和校验函数读取 `this`，但在 Vue 3 中这些函数不能依赖组件实例。

```vue annotate="remove:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// `this` is not the component instance in Vue 3.
default() {
return this.defaultSize
}
},
value: {
type: Number,
validator() {
return this.value > 0
}
}
}
}
</script>
```

<span id="script-no-deprecated-props-default-this-good"></span>

**正确示例**

默认值从参数读取 `props.baseSize`，校验函数检查 `value` 参数。两者都不再依赖不可用的实例接收对象。

```vue annotate="add:6,7,8,13,14"
<script lang="ts">
export default {
props: {
size: {
type: Number,
// Vue 3 passes the raw props as the first argument instead.
default(props) {
return props.baseSize
}
},
value: {
type: Number,
validator(value) {
return value > 0
}
}
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_deprecated_props_default_this.rs#L71) · [全部规则](all.md)

### `script/no-dupe-keys`

禁止 Options API 的 props/data/computed/methods/setup/inject 之间重复的键

[错误示例](#script-no-dupe-keys-bad) · [正确示例](#script-no-dupe-keys-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-dupe-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-dupe-keys-bad"></span>

**错误示例**

props 和 data 都声明 `foo`，computed 和 methods 都声明 `bar`。这些声明竞争相同的组件实例键。

```vue annotate="remove:5,8,9,10,11"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { foo: 1 } // duplicate of prop `foo`
},
computed: {
bar() { return 2 }
},
methods: {
bar() {} // duplicate of computed `bar`
}
}
</script>
```

<span id="script-no-dupe-keys-good"></span>

**正确示例**

prop、data 和 computed 声明使用不同名称（`foo`、`bar` 和 `baz`），消除两处跨选项冲突。

```vue annotate="add:5,8"
<script lang="ts">
export default {
props: ['foo'],
data() {
return { bar: 1 }
},
computed: {
baz() { return 2 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_dupe_keys.rs#L52) · [全部规则](all.md)

### `script/no-duplicate-attr-inheritance`

标记重复应用透传属性的组件

[错误示例](#script-no-duplicate-attr-inheritance-bad) · [正确示例](#script-no-duplicate-attr-inheritance-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-duplicate-attr-inheritance": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-duplicate-attr-inheritance-bad"></span>

**错误示例**

显式 `inheritAttrs: true` 重复了 Vue 的默认值。即使没有展示根节点 `$attrs` 展开，规则也会报告此多余字面量。

```vue annotate="remove:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: true })
export default { inheritAttrs: true }
</script>
```

<span id="script-no-duplicate-attr-inheritance-good"></span>

**正确示例**

`inheritAttrs: false` 表达实际的退出选择，空选项对象则隐式保留默认继承。两者都不重复多余的 `true`。

```vue annotate="add:2,3"
<script lang="ts">
defineOptions({ inheritAttrs: false }) // intentional opt-out
export default {}                      // default inheritance, unstated
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_duplicate_attr_inheritance.rs#L73) · [全部规则](all.md)

### `script/no-export-in-script-setup`

禁止 &lt;script setup&gt; 内的 export 语句

[错误示例](#script-no-export-in-script-setup-bad) · [正确示例](#script-no-export-in-script-setup-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-export-in-script-setup": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-export-in-script-setup-bad"></span>

**错误示例**

`export const count` 试图从 `<script setup>` 暴露模块导出，但这里禁止运行时导出。

```vue annotate="remove:2"
<script setup lang="ts">
export const count = 1;
</script>
```

<span id="script-no-export-in-script-setup-good"></span>

**正确示例**

移除 `export`，使 `count` 保持为 setup 绑定，而非模块导出。

```vue annotate="add:2"
<script setup lang="ts">
const count = 1;
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_export_in_script_setup.rs#L49) · [全部规则](all.md)

### `script/no-get-current-instance`

禁止在 Vapor 模式使用 getCurrentInstance()（返回 null）

[错误示例](#script-no-get-current-instance-bad) · [正确示例](#script-no-get-current-instance-good)

默认严重程度: `error`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-get-current-instance": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-get-current-instance-bad"></span>

**错误示例**

带 Vapor 标记的 setup 导入并调用 `getCurrentInstance`，依赖了此规则针对 Vapor 组件禁止的实例 API。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { getCurrentInstance } from "vue";
const instance = getCurrentInstance();
</script>
```

<span id="script-no-get-current-instance-good"></span>

**正确示例**

`inject("app-config")` 获取显式提供的配置，无需导入或调用 `getCurrentInstance`。

```vue annotate="add:2,3"
<script setup lang="ts" vapor>
import { inject } from "vue";
const appConfig = inject("app-config");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_get_current_instance.rs#L38) · [全部规则](all.md)

### `script/no-import-compiler-macros`

禁止导入自动可用的 Vue 编译器宏

[错误示例](#script-no-import-compiler-macros-bad) · [正确示例](#script-no-import-compiler-macros-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-import-compiler-macros": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-import-compiler-macros-bad"></span>

**错误示例**

`vue` 导入包含 `defineProps` 和 `defineEmits`，但这些编译器宏已在 `<script setup>` 中直接可用。

```vue annotate="remove:2"
<script setup lang="ts">
import { defineProps, defineEmits } from "vue";
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

<span id="script-no-import-compiler-macros-good"></span>

**正确示例**

移除宏导入，保留两个带类型的宏调用；两种声明都无需运行时导入。

```vue
<script setup lang="ts">
const props = defineProps<{ title: string }>();
const emit = defineEmits<{ save: [id: number] }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_import_compiler_macros.rs#L39) · [全部规则](all.md)

### `script/no-internal-imports`

禁止从 Vue 内部模块导入

[错误示例](#script-no-internal-imports-bad) · [正确示例](#script-no-internal-imports-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-internal-imports": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-internal-imports-bad"></span>

**错误示例**

两个导入都指向内部 `dist` 文件，而非 Vue 公共包入口，使组件与构建文件路径耦合。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { foo } from '@vue/runtime-core/dist/runtime-core.esm-bundler'
import { bar } from 'vue/dist/vue.esm-bundler'
</script>
```

<span id="script-no-internal-imports-good"></span>

**正确示例**

从 `vue` 导入所需辅助函数，移除对内部发行文件位置的依赖。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, computed } from 'vue'
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_internal_imports.rs#L28) · [全部规则](all.md)

### `script/no-multiple-slot-args`

禁止向作用域插槽函数调用传入多个参数

[错误示例](#script-no-multiple-slot-args-bad) · [正确示例](#script-no-multiple-slot-args-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-multiple-slot-args": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-multiple-slot-args-bad"></span>

**错误示例**

插槽调用传入多个位置参数，或展开未知参数列表。Vue 插槽接收一个 props 对象，而非位置参数列表。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
slots.default(foo, bar)
$slots.header(a, b)
this.$scopedSlots.item(x, y)
useSlots().default(a, b)
slots.default(...args)
</script>
```

<span id="script-no-multiple-slot-args-good"></span>

**正确示例**

`{ foo, bar }` 将数据合并为一个参数；`slotProps` 和无参数调用也符合支持的插槽调用形式。

```vue annotate="add:2,3,4"
<script setup lang="ts">
slots.default({ foo, bar })
slots.default(slotProps)
slots.default()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_multiple_slot_args.rs#L61) · [全部规则](all.md)

### `script/no-next-tick`

禁止面向 Vapor 的组件使用 nextTick()

[错误示例](#script-no-next-tick-bad) · [正确示例](#script-no-next-tick-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-next-tick": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-next-tick-bad"></span>

**错误示例**

面向 Vapor 的组件导入并等待 `nextTick`，引入此迁移规则拒绝的 DOM 刷新调度依赖。

```vue annotate="remove:2,3"
<script setup lang="ts" vapor>
import { nextTick } from "vue";
await nextTick();
</script>
```

<span id="script-no-next-tick-good"></span>

**正确示例**

通过 `useTemplateRef` 获取输入框，并在 `onMounted` 时聚焦。显式挂载边界替代示例中的 `nextTick` 依赖。

```vue annotate="add:2,3,4,6"
<script setup lang="ts" vapor>
import { onMounted, useTemplateRef } from "vue";
const input = useTemplateRef<HTMLInputElement>("input");
onMounted(() => { input.value?.focus(); });
</script>
<template><input ref="input"></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_next_tick.rs#L40) · [全部规则](all.md)

### `script/no-options-api`

禁止 Vapor 模式中的 Options API 模式

[错误示例](#script-no-options-api-bad) · [正确示例](#script-no-options-api-good)

默认严重程度: `error`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: 面向 Vapor 的脚本检查；显式启用后，限制也适用于普通脚本  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-options-api": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-options-api-bad"></span>

**错误示例**

默认导出对象声明了 Options API 的 `data()`，这是此规则禁止的组件选项形式。

```vue annotate="remove:1,2,3,4,5,6"
<script lang="ts">
export default {
  data() {
    return { count: 0 };
  },
};
</script>
```

<span id="script-no-options-api-good"></span>

**正确示例**

组件状态改为 Vapor `<script setup>` 中的 Composition API `ref`，移除 Options API 对象及其 `data` 选项。

```vue annotate="add:1,2"
<script setup lang="ts" vapor>
const count = ref(0);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_options_api.rs#L43) · [全部规则](all.md)

### `script/no-potential-component-option-typo`

标记 Options API 组件选项名称中的疑似拼写错误

[错误示例](#script-no-potential-component-option-typo-bad) · [正确示例](#script-no-potential-component-option-typo-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-potential-component-option-typo": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-potential-component-option-typo-bad"></span>

**错误示例**

选项被写为 `method`，与受识别的 `methods` 选项只差一次编辑；Vue 不会将它视为预期的方法声明。

```vue annotate="remove:2"
<script lang="ts">
export default { method: { save() {} } };
</script>
```

<span id="script-no-potential-component-option-typo-good"></span>

**正确示例**

将键改为 `methods`，使 `save()` 位于受识别的组件选项下。

```vue annotate="add:2"
<script lang="ts">
export default { methods: { save() {} } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_potential_component_option_typo.rs#L19) · [全部规则](all.md)

### `script/no-reactive-destructure`

禁止导致响应性丢失的响应式对象解构

[错误示例](#script-no-reactive-destructure-bad) · [正确示例](#script-no-reactive-destructure-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reactive-destructure": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reactive-destructure-bad"></span>

**错误示例**

`const { count, name } = state` 从 `reactive` 对象复制原始类型属性，失去与后续属性变化的联系。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = state;
</script>
```

<span id="script-no-reactive-destructure-good"></span>

**正确示例**

解构 `toRefs(state)` 为 `count` 和 `name` 创建 refs，使每个绑定继续关联原始响应式属性。

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRefs } from "vue";
const state = reactive({ count: 0, name: "Ada" });
const { count, name } = toRefs(state);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reactive_destructure.rs#L43) · [全部规则](all.md)

### `script/no-ref-as-operand`

要求将 ref 绑定变量用作操作数时通过 `.value` 访问

[错误示例](#script-no-ref-as-operand-bad) · [正确示例](#script-no-ref-as-operand-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-ref-as-operand": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-ref-as-operand-bad"></span>

**错误示例**

`count + 1` 将 ref 对象本身用作算术操作数，而不是其包裹的数字。

```vue annotate="remove:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count + 1;
</script>
```

<span id="script-no-ref-as-operand-good"></span>

**正确示例**

`count.value + 1` 先读取包裹的数字，再加一；脚本算术需要此显式 ref 访问。

```vue annotate="add:4"
<script setup lang="ts">
import { ref } from "vue";
const count = ref(0);
const next = count.value + 1;
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_ref_as_operand.rs#L41) · [全部规则](all.md)

### `script/no-required-prop-with-default`

禁止 prop 同时具有 required: true 和默认值

[错误示例](#script-no-required-prop-with-default-bad) · [正确示例](#script-no-required-prop-with-default-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-required-prop-with-default": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-required-prop-with-default-bad"></span>

**错误示例**

`title` 既是必需项，又具有后备值 `"Untitled"`，混合了必需输入约定和面向缺失输入的默认值。

```vue annotate="remove:2"
<script lang="ts">
export default { props: { title: { type: String, required: true, default: "Untitled" } } };
</script>
```

<span id="script-no-required-prop-with-default-good"></span>

**正确示例**

移除 `required: true`，使 `title` 可选，并保留 `"Untitled"` 作为一致的后备值。

```vue annotate="add:2"
<script lang="ts">
export default { props: { title: { type: String, default: "Untitled" } } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_required_prop_with_default.rs#L29) · [全部规则](all.md)

### `script/no-reserved-identifiers`

禁止使用 Vue 编译器保留标识符

[错误示例](#script-no-reserved-identifiers-bad) · [正确示例](#script-no-reserved-identifiers-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-identifiers": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-identifiers-bad"></span>

**错误示例**

绑定 `__props`、`__emit` 和 `__sfc__` 使用了为 Vue 编译器生成代码保留的标识符。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const __props = { name: "Ada" };
const __emit = () => {};
const __sfc__ = {};
</script>
```

<span id="script-no-reserved-identifiers-good"></span>

**正确示例**

普通名称 `props`、`emit` 和 `componentData` 避开这些生成标识符，同时保留 props 和 emits 声明。

```vue annotate="add:2,3,4"
<script setup lang="ts">
const props = defineProps<{ name: string }>();
const emit = defineEmits<{ save: [] }>();
const componentData = {};
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_identifiers.rs#L49) · [全部规则](all.md)

### `script/no-reserved-keys`

禁止将 Vue 保留名称用作 Options API props/data/computed/methods/setup/inject 键

[错误示例](#script-no-reserved-keys-bad) · [正确示例](#script-no-reserved-keys-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-keys": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-keys-bad"></span>

**错误示例**

返回的 data 键 `$el` 与 Vue 内置组件实例属性冲突，也使用了保留的 `$` 前缀。

```vue annotate="remove:2"
<script lang="ts">
export default { data() { return { $el: "custom" }; } };
</script>
```

<span id="script-no-reserved-keys-good"></span>

**正确示例**

将应用数据重命名为 `elementLabel`，避开内置实例接口和保留前缀。

```vue annotate="add:2"
<script lang="ts">
export default { data() { return { elementLabel: "custom" }; } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_reserved_keys.rs#L32) · [全部规则](all.md)

### `script/no-reserved-props`

禁止在组件 props 声明中使用保留名称

[错误示例](#script-no-reserved-props-bad) · [正确示例](#script-no-reserved-props-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-reserved-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-reserved-props-bad"></span>

**错误示例**

对象形式的 `ref` 和 `$foo`，以及数组形式的 `key`，都是保留 prop 名称。`ref` 和 `key` 是框架控制项，带 `$` 前缀的名称也会被拒绝。

```vue annotate="remove:4,5,6,8,9,10"
<script lang="ts">
export default {
props: {
ref: String,   // reserved
$foo: Number    // `$`-prefixed names are reserved
}
}

export default {
props: ['key']    // reserved (array form)
}
</script>
```

<span id="script-no-reserved-props-good"></span>

**正确示例**

普通 prop 名称 `name` 和 `refValue` 在拼写及前缀上都避开了保留名称。

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
name: String,
refValue: Number
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_reserved_props.rs#L53) · [全部规则](all.md)

### `script/no-restricted-globals`

禁止直接引用必须通过类型化包装器访问的运行时环境全局变量

[错误示例](#script-no-restricted-globals-bad) · [正确示例](#script-no-restricted-globals-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-globals": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-globals-bad"></span>

**错误示例**

示例直接读取默认受限全局变量 `process`、`localStorage` 和 `sessionStorage`，绕过项目显式的配置和存储辅助函数。

```vue annotate="remove:2,3,4"
<script setup lang="ts">
const flag = process.env.FEATURE_FLAG
const token = localStorage.getItem('auth.token')
sessionStorage.setItem('view.scroll', String(window.scrollY))
</script>
```

<span id="script-no-restricted-globals-good"></span>

**正确示例**

`useFeatureFlag`、`authStorage.read` 和 `viewStorage.write` 移除了这些直接引用。剩余的 `window.scrollY` 不是规则的默认限制；SSR 安全是独立问题。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
// Use a typed config helper that distinguishes server vs. client.
const flag = useFeatureFlag('FEATURE_FLAG')

// Use a typed wrapper that scopes keys and handles SSR / disabled storage.
const token = authStorage.read('auth.token')
viewStorage.write('view.scroll', String(window.scrollY))
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_globals.rs#L57) · [全部规则](all.md)

### `script/no-restricted-members`

禁止项目配置的 object.property 成员访问

[错误示例](#script-no-restricted-members-bad) · [正确示例](#script-no-restricted-members-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 参见[类型化选项和默认值](/rules/options.md)。

此示例配置 window.localStorage。规则没有默认拒绝列表；仅启用规则不会报告成员。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-restricted-members": "error"
      },
      "ruleOptions": {
        "script/no-restricted-members": {
          "members": [
            {
              "object": "window",
              "property": "localStorage"
            }
          ]
        }
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-restricted-members-bad"></span>

**错误示例**

在 `ruleOptions` 中配置 `{ object: "window", property: "localStorage" }` 后，`window.localStorage` 访问了禁止的对象与成员组合。规则没有默认禁止成员。

```vue annotate="remove:2"
<script setup lang="ts">
const token = window.localStorage.getItem("token");
</script>
```

<span id="script-no-restricted-members-good"></span>

**正确示例**

`authStorage.read("token")` 将读取交给应用存储辅助函数，不再访问配置禁止的 `window.localStorage` 成员。

```vue annotate="add:2"
<script setup lang="ts">
const token = authStorage.read("token");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_restricted_members.rs#L53) · [全部规则](all.md)

### `script/no-side-effects-in-computed-properties`

禁止 Options API 计算 getter 中的副作用

[错误示例](#script-no-side-effects-in-computed-properties-bad) · [正确示例](#script-no-side-effects-in-computed-properties-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-side-effects-in-computed-properties": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-side-effects-in-computed-properties-bad"></span>

**错误示例**

`doubled` 赋值给 `this.count`，`reversed` 通过 `reverse()` 修改 `this.items`。两个 getter 都修改了本应只用于派生的状态。

```vue annotate="remove:8,9,12"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
this.count = this.count * 2 // side effect: assigns to data
return this.count
},
reversed() {
return this.items.reverse() // side effect: mutates the array
}
}
}
</script>
```

<span id="script-no-side-effects-in-computed-properties-good"></span>

**正确示例**

`doubled` 返回乘法结果，不进行赋值。`reversed` 先复制数组再反转，getter 不会改变原组件状态。

```vue annotate="add:8,11"
<script lang="ts">
export default {
data() {
return { count: 0, items: [] }
},
computed: {
doubled() {
return this.count * 2
},
reversed() {
return [...this.items].reverse() // operate on a copy
}
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_side_effects_in_computed.rs#L70) · [全部规则](all.md)

### `script/no-top-level-ref-in-script`

禁止顶层 ref/reactive，以防止跨请求状态污染

[错误示例](#script-no-top-level-ref-in-script-bad) · [正确示例](#script-no-top-level-ref-in-script-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-top-level-ref-in-script": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-top-level-ref-in-script-bad"></span>

**错误示例**

普通 `<script>` 在模块作用域初始化 `count` 和 `user`。SSR 期间，这些状态对象可能跨组件实例和请求共享。

```vue annotate="remove:1,2,4,8"
<script>
// This state is shared across all requests in SSR!
const count = ref(0)
const user = reactive({ name: '' })

export default {
setup() {
return { count, user }
}
}
</script>
```

<span id="script-no-top-level-ref-in-script-good"></span>

**正确示例**

setup ref 按组件实例初始化；普通脚本只保留常量、生成状态的函数，以及 `setup()` 内创建的 ref。都不会在普通模块作用域创建响应式状态。

```vue annotate="add:1,2,4,6,7,8,9,10,11,12,13,14,17,18,19"
<script setup>
// Script setup creates fresh state per request
const count = ref(0)
</script>

<script>
// Constants are fine
const API_URL = 'https://api.example.com'

// Functions that create state are fine
function createState() {
return reactive({ count: 0 })
}

export default {
setup() {
// Create state inside setup
const count = ref(0)
return { count }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_top_level_ref_in_script.rs#L63) · [全部规则](all.md)

### `script/no-unstable-nested-components`

禁止在 setup 或 render 函数内定义组件

[错误示例](#script-no-unstable-nested-components-bad) · [正确示例](#script-no-unstable-nested-components-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unstable-nested-components": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unstable-nested-components-bad"></span>

**错误示例**

`defineComponent` 在父组件的 `setup()` 内运行，每次执行 setup 都创建新的 `Child` 组件定义。

```vue annotate="remove:3"
<script lang="ts">
import { defineComponent } from "vue";
export default { setup() { const Child = defineComponent({ render() { return null; } }); return { Child }; } };
</script>
```

<span id="script-no-unstable-nested-components-good"></span>

**正确示例**

`Child` 定义移到模块作用域，`setup()` 返回已存在的定义。

```vue annotate="add:3,4"
<script lang="ts">
import { defineComponent } from "vue";
const Child = defineComponent({ render() { return null; } });
export default { setup() { return { Child }; } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_unstable_nested_components.rs#L19) · [全部规则](all.md)

### `script/no-unused-emit-declarations`

标记已声明却从未发出的事件

[错误示例](#script-no-unused-emit-declarations-bad) · [正确示例](#script-no-unused-emit-declarations-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-unused-emit-declarations": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-unused-emit-declarations-bad"></span>

**错误示例**

`defineEmits` 声明了 `change` 和 `unused`，但捕获的 `emit` 函数只发出字面量事件 `change`。

```vue annotate="remove:2,4"
<script setup lang="ts">
const emit = defineEmits(['change', 'unused'])
emit('change')
// `unused` is never emitted
</script>
```

<span id="script-no-unused-emit-declarations-good"></span>

**正确示例**

移除 `unused`，使声明的事件列表与观察到的发出操作一致。示例使用捕获且未逸出的 emit 绑定，因此可以得出此局部使用结论。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(['change'])
emit('change')
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/no_unused_emit_declarations.rs#L74) · [全部规则](all.md)

### `script/no-use-computed-property-like-method`

禁止像方法一样调用 Options API 计算属性

[错误示例](#script-no-use-computed-property-like-method-bad) · [正确示例](#script-no-use-computed-property-like-method-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-use-computed-property-like-method": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-use-computed-property-like-method-bad"></span>

**错误示例**

`this.total()` 调用了计算 getter 暴露的值；getter 返回不可调用的 `3`。

```vue annotate="remove:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total()); } } };
</script>
```

<span id="script-no-use-computed-property-like-method-good"></span>

**正确示例**

`this.total` 不加调用括号，直接读取计算值，使 `log` 打印派生数字。

```vue annotate="add:2"
<script lang="ts">
export default { computed: { total() { return 3; } }, methods: { log() { console.log(this.total); } } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_use_computed_property_like_method.rs#L44) · [全部规则](all.md)

### `script/no-with-defaults`

不建议使用 withDefaults，优先使用解构默认值（Vue 3.5+）

[错误示例](#script-no-with-defaults-bad) · [正确示例](#script-no-with-defaults-good)

默认严重程度: `warning`  
预设: `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/no-with-defaults": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-no-with-defaults-bad"></span>

**错误示例**

`withDefaults` 包装带类型的 props 声明，仅为 `count` 和 `name` 提供默认值，没有使用此处偏好的 Vue 3.5+ 解构默认值风格。

```vue annotate="remove:2"
<script setup lang="ts">
const props = withDefaults(defineProps<{ count?: number; name?: string }>(), { count: 0, name: "Ada" });
</script>
```

<span id="script-no-with-defaults-good"></span>

**正确示例**

解构模式将 `count = 0` 和 `name = "Ada"` 放在对应绑定旁，并移除 `withDefaults` 包装。

```vue annotate="add:2"
<script setup lang="ts">
const { count = 0, name = "Ada" } = defineProps<{ count?: number; name?: string }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/no_with_defaults.rs#L41) · [全部规则](all.md)

### `script/prefer-computed`

响应式派生状态优先使用 computed()

[错误示例](#script-prefer-computed-bad) · [正确示例](#script-prefer-computed-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

此情形要求侦听器仅派生目标值。可编辑副本及具有其他副作用的回调被允许。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-computed": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-computed-bad"></span>

**错误示例**

侦听器仅将 `count` 的派生值复制到第二个 ref `doubled`，通过手动同步维护派生状态。

```vue annotate="remove:2,4,5"
<script setup lang="ts">
import { ref, watch } from "vue";
const count = ref(0);
const doubled = ref(0);
watch(count, (value) => { doubled.value = value * 2; });
</script>
```

<span id="script-prefer-computed-good"></span>

**正确示例**

`computed(() => count.value * 2)` 直接表达派生关系，移除额外可写 ref 及其同步侦听器。

```vue annotate="add:2,4"
<script setup lang="ts">
import { ref, computed } from "vue";
const count = ref(0);
const doubled = computed(() => count.value * 2);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_computed.rs#L41) · [全部规则](all.md)

### `script/prefer-define-options`

优先使用 defineOptions()，而不是仅设置 name/inheritAttrs 的普通 &lt;script&gt;

[错误示例](#script-prefer-define-options-bad) · [正确示例](#script-prefer-define-options-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-define-options": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-define-options-bad"></span>

**错误示例**

普通脚本唯一有实质作用的语句导出只含 `name` 和 `inheritAttrs` 的对象；这些选项可通过 `defineOptions` 表达。

```vue annotate="remove:2"
<script lang="ts">
export default { name: 'MyComponent', inheritAttrs: false }
</script>
```

<span id="script-prefer-define-options-good"></span>

**正确示例**

示例中的 `data()` 方法让脚本包含实际 Options API 逻辑，因此不在此规则保守的仅选项建议范围内。此正确示例展示允许的例外；直接迁移则应在 `<script setup>` 中使用 `defineOptions({ name: 'MyComponent', inheritAttrs: false })`。

```vue annotate="add:2,3,4,5,6"
<script lang="ts">
// Real options logic — keep the plain script.
export default {
name: 'MyComponent',
data() { return { count: 0 } },
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_define_options.rs#L52) · [全部规则](all.md)

### `script/prefer-import-from-vue`

优先从 'vue' 导入，而不是内部包

[错误示例](#script-prefer-import-from-vue-bad) · [正确示例](#script-prefer-import-from-vue-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 适用于已支持的诊断  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-import-from-vue": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-import-from-vue-bad"></span>

**错误示例**

`ref` 和 `h` 从内部 `@vue/runtime-core` 和 `@vue/runtime-dom` 包导入，而不是公共 `vue` 包。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from '@vue/runtime-core'
import { h } from '@vue/runtime-dom'
</script>
```

<span id="script-prefer-import-from-vue-good"></span>

**正确示例**

两个辅助函数一起从 `vue` 导入，使用公共包入口，不使用任何内部包。

```vue annotate="add:2"
<script setup lang="ts">
import { ref, h } from 'vue'
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_import_from_vue.rs#L32) · [全部规则](all.md)

### `script/prefer-ref-over-reactive`

建议使用 ref() 而不是 reactive() 管理状态

[错误示例](#script-prefer-ref-over-reactive-bad) · [正确示例](#script-prefer-ref-over-reactive-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-ref-over-reactive": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-ref-over-reactive-bad"></span>

**错误示例**

状态通过 `reactive` 创建，不符合此主张型规则对 refs 的偏好。示例展示风格偏好，并非响应式对象本身无效。

```vue annotate="remove:2,3,4,5,6"
<script setup lang="ts">
// reactive requires careful handling to avoid losing reactivity
const state = reactive({
count: 0,
name: 'foo'
})
</script>
```

<span id="script-prefer-ref-over-reactive-good"></span>

**正确示例**

示例通过 `ref` 创建标量和对象状态；相关字段也可拆为独立 refs，满足偏好的状态构造形式。

```vue annotate="add:2,3,4,5,6,7,8,9,10,11"
<script setup lang="ts">
// ref is more explicit and safer
const count = ref(0)
const name = ref('foo')

// For objects, ref still works
const user = ref({ name: 'foo', age: 20 })

// Or use multiple refs for related data
const userName = ref('foo')
const userAge = ref(20)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_ref_over_reactive.rs#L44) · [全部规则](all.md)

### `script/prefer-use-attrs`

建议使用 useAttrs() 而不是 context.attrs

[错误示例](#script-prefer-use-attrs-bad) · [正确示例](#script-prefer-use-attrs-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-attrs": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-attrs-bad"></span>

**错误示例**

`setup` 通过解构上下文参数获取 `attrs`，规则要求改用 Composition API 辅助函数。

```vue annotate="remove:2"
<script lang="ts">
export default { setup(_props, { attrs }) { console.log(attrs.class); } };
</script>
```

<span id="script-prefer-use-attrs-good"></span>

**正确示例**

`useAttrs()` 在 setup 内提供 `attrs`，保留 `attrs.class` 读取，不依赖第二个 setup 参数。

```vue annotate="add:2,3"
<script lang="ts">
import { useAttrs } from "vue";
export default { setup() { const attrs = useAttrs(); console.log(attrs.class); } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_attrs.rs#L44) · [全部规则](all.md)

### `script/prefer-use-id`

建议使用 useId() 生成唯一 ID（Vue 3.5+）

[错误示例](#script-prefer-use-id-bad) · [正确示例](#script-prefer-use-id-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-id": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-id-bad"></span>

**错误示例**

`id` 包含 `Math.random()`，生成的输入框与标签标识符可能在服务端和客户端渲染时不同。以 ID 命名的绑定是规则可识别的生成上下文。

```vue annotate="remove:2"
<script setup lang="ts">
const id = `input-${Math.random()}`;
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

<span id="script-prefer-use-id-good"></span>

**正确示例**

Vue 3.5+ 的 `useId()` 生成标识符，`:for` 和 `:id` 继续读取同一绑定，不再分别生成随机值。

```vue annotate="add:2,3"
<script setup lang="ts">
import { useId } from "vue";
const id = useId();
</script>
<template><label :for="id">Name</label><input :id="id" /></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_id.rs#L48) · [全部规则](all.md)

### `script/prefer-use-slots`

建议使用 useSlots() 而不是 context.slots

[错误示例](#script-prefer-use-slots-bad) · [正确示例](#script-prefer-use-slots-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-slots-bad"></span>

**错误示例**

`setup` 从上下文参数解构 `slots`，这是此规则建议替换的访问形式。

```vue annotate="remove:2,4"
<script lang="ts">
import { defineComponent, h } from "vue";
export default defineComponent({
  setup(_props, { slots }) { return () => h("div", slots.default?.()); },
});
</script>
```

<span id="script-prefer-use-slots-good"></span>

**正确示例**

`useSlots()` 在 setup 内获取插槽，保留渲染函数及可选默认插槽调用，不使用上下文参数。

```vue annotate="add:2,4,5,6,7"
<script lang="ts">
import { defineComponent, h, useSlots } from "vue";
export default defineComponent({
  setup() {
    const slots = useSlots();
    return () => h("div", slots.default?.());
  },
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_slots.rs#L44) · [全部规则](all.md)

### `script/prefer-use-template-ref`

模板引用建议使用 useTemplateRef 而不是 ref（Vue 3.5+）

[错误示例](#script-prefer-use-template-ref-bad) · [正确示例](#script-prefer-use-template-ref-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/prefer-use-template-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-prefer-use-template-ref-bad"></span>

**错误示例**

可为空的 `input` ref 与模板字面量 `ref="input"` 配对，表明它是元素引用，而非普通可空数据。

```vue annotate="remove:2,3"
<script setup lang="ts">
import { ref } from 'vue'
const input = ref<HTMLInputElement | null>(null)
</script>
<template>
<input ref="input" />
</template>
```

<span id="script-prefer-use-template-ref-good"></span>

**正确示例**

Vue 3.5+ 的 `useTemplateRef<HTMLInputElement>('input')` 显式表达模板引用。未配对的 `error = ref(null)` 仍是普通数据，有意不在此规则范围内。

```vue annotate="add:2,3,4,5,6,10"
<script setup lang="ts">
import { ref, useTemplateRef } from 'vue'
// Paired with the template ref below.
const input = useTemplateRef<HTMLInputElement>('input')
// A nullable data ref the template never binds as a ref.
const error = ref(null)
</script>
<template>
<input ref="input" />
<p>{{ error }}</p>
</template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/prefer_use_template_ref.rs#L75) · [全部规则](all.md)

### `script/require-default-prop`

要求每个可选的非 Boolean prop 具有默认值

[错误示例](#script-require-default-prop-bad) · [正确示例](#script-require-default-prop-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-default-prop-bad"></span>

**错误示例**

`name` 和 `age` 是没有默认值的可选非 Boolean 运行时 props，省略输入时的值未指定。

```vue annotate="remove:4,5,6"
<script lang="ts">
export default {
props: {
// optional, non-Boolean, no default
name: String,
age: { type: Number },
}
}
</script>
```

<span id="script-require-default-prop-good"></span>

**正确示例**

`name` 获得 `default: ''`。`enabled` 使用 Boolean 隐式的 false 默认值，必需的 `id` 无需后备值，展示了两种豁免。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: { type: String, default: '' },
enabled: Boolean,                 // Boolean defaults to false
id: { type: Number, required: true },
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_default_prop.rs#L58) · [全部规则](all.md)

### `script/require-explicit-emits`

要求发出的事件在 defineEmits 或 emits 选项中声明

[错误示例](#script-require-explicit-emits-bad) · [正确示例](#script-require-explicit-emits-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-emits": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-emits-bad"></span>

**错误示例**

捕获的 emit 函数发出 `save`，但 `defineEmits([])` 未声明该事件。

```vue annotate="remove:2"
<script setup lang="ts">
const emit = defineEmits([]);
emit("save");
</script>
```

<span id="script-require-explicit-emits-good"></span>

**正确示例**

在声明中添加 `"save"`，使发出的字面量事件成为组件显式事件约定的一部分。

```vue annotate="add:2"
<script setup lang="ts">
const emit = defineEmits(["save"]);
emit("save");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_explicit_emits.rs#L61) · [全部规则](all.md)

### `script/require-explicit-slots`

要求通过 useSlots() 使用的插槽由 defineSlots&lt;...&gt;() 显式定义类型

[错误示例](#script-require-explicit-slots-bad) · [正确示例](#script-require-explicit-slots-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-explicit-slots": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-explicit-slots-bad"></span>

**错误示例**

带类型的 `defineProps<{ id: number }>()` 确立了 TypeScript 语法，但 setup 使用 `useSlots()` 而没有 `defineSlots` 声明，因此规则发现缺少显式插槽约定。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps<{ id: number }>()
const slots = useSlots()
</script>
```

<span id="script-require-explicit-slots-good"></span>

**正确示例**

`defineSlots` 声明 props 含 `msg: string` 的 `default` 插槽；`useSlots()` 现在配有显式的类型化插槽约定。

```vue annotate="add:2"
<script setup lang="ts">
defineSlots<{ default(props: { msg: string }): unknown }>()
const slots = useSlots()
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_explicit_slots.rs#L92) · [全部规则](all.md)

### `script/require-function-return-type`

要求函数具有返回类型注解

[错误示例](#script-require-function-return-type-bad) · [正确示例](#script-require-function-return-type-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-function-return-type": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-function-return-type-bad"></span>

**错误示例**

`add` 和 `greet` 都注解了参数，却省略返回类型注解；推断返回类型不满足此显式注解策略。

```vue annotate="remove:2,6"
<script setup lang="ts">
const add = (a: number, b: number) => {
return a + b
}

function greet(name: string) {
return `Hello, ${name}`
}
</script>
```

<span id="script-require-function-return-type-good"></span>

**正确示例**

`add` 声明 `: number`，`greet` 声明 `: string`，明确返回约定而不改变函数体。

```vue annotate="add:2,6"
<script setup lang="ts">
const add = (a: number, b: number): number => {
return a + b
}

function greet(name: string): string {
return `Hello, ${name}`
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_function_return_type.rs#L49) · [全部规则](all.md)

### `script/require-prop-type-constructor`

要求 prop 的 `type` 值是构造器，而不是字符串字面量

[错误示例](#script-require-prop-type-constructor-bad) · [正确示例](#script-require-prop-type-constructor-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-type-constructor": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-type-constructor-bad"></span>

**错误示例**

prop 声明将字符串 `"String"` 和 `"Number"` 用作运行时类型，包括构造器数组内的值。这些字符串不是构造函数。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
// The type should be the `String` constructor, not the string "String".
name: "String",
age: { type: "Number" },
id: { type: ["String", "Number"] }
}
}
</script>
```

<span id="script-require-prop-type-constructor-good"></span>

**正确示例**

声明使用实际的 `String` 和 `Number` 标识符，包括联合数组 `[String, Number]`。

```vue annotate="add:4,5,6"
<script lang="ts">
export default {
props: {
name: String,
age: { type: Number },
id: { type: [String, Number] }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_prop_type_constructor.rs#L59) · [全部规则](all.md)

### `script/require-prop-types`

要求每个 prop 声明类型

[错误示例](#script-require-prop-types-bad) · [正确示例](#script-require-prop-types-good)

默认严重程度: `error`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-prop-types": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-prop-types-bad"></span>

**错误示例**

数组项只声明名称 `status`；`null` 值和空描述对象也没有声明运行时 prop 类型。

```vue annotate="remove:3,4,5,6,8,9"
<script lang="ts">
export default {
props: ['status']            // array form: no types
}

export default {
props: {
status: null,              // no type
other: {}                  // empty descriptor: no type
}
}
</script>
```

<span id="script-require-prop-types-good"></span>

**正确示例**

`status: String` 提供简写构造器，`other` 在描述对象内提供 `type: Number`，两个 props 现在都有类型声明。

```vue annotate="add:4,5"
<script lang="ts">
export default {
props: {
status: String,
other: { type: Number, default: 0 }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_prop_types.rs#L58) · [全部规则](all.md)

### `script/require-symbol-provide`

建议使用 Symbol 作为 provide/inject 的注入键

[错误示例](#script-require-symbol-provide-bad) · [正确示例](#script-require-symbol-provide-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-symbol-provide": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-symbol-provide-bad"></span>

**错误示例**

`provide` 和 `inject` 使用 `'user'`、`'theme'` 等字符串字面量键，可能与同名的其他提供者冲突。

```vue annotate="remove:1,2,3,4,6,7"
<script setup lang="ts">
// String keys can collide
provide('user', user)
const user = inject('user')

// Magic strings are error-prone
provide('theme', { dark: true })
</script>
```

<span id="script-require-symbol-provide-good"></span>

**正确示例**

共享的 `UserKey` 通过 `Symbol` 创建，并注解为 `InjectionKey<User>`；两个调用都传入该键，而非字符串字面量。

```vue annotate="add:1,2,3,5,6,7,8,9"
<script lang="ts">
// Define injection key with Symbol
export const UserKey: InjectionKey<User> = Symbol('user')

// Provide with Symbol
provide(UserKey, user)

// Inject with Symbol
const user = inject(UserKey)
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_symbol_provide.rs#L39) · [全部规则](all.md)

### `script/require-typed-object-prop`

要求运行时类型为 `Object` 或 `Array` 的 prop 具有显式类型

[错误示例](#script-require-typed-object-prop-bad) · [正确示例](#script-require-typed-object-prop-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-object-prop": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-object-prop-bad"></span>

**错误示例**

单独的 `Object` 和 `Array` 构造器只描述宽泛的运行时类别，因此 `user` 和 `items` 元素结构都没有显式静态类型。

```vue annotate="remove:2"
<script setup lang="ts">
const props = defineProps({ user: Object, items: { type: Array } });
</script>
```

<span id="script-require-typed-object-prop-good"></span>

**正确示例**

`PropType<User>` 和 `PropType<User[]>` 添加对象和元素类型，同时保留原运行时构造器。

```vue annotate="add:2,3,4,5,6,7"
<script setup lang="ts">
import type { PropType } from "vue";
interface User { name: string }
const props = defineProps({
  user: Object as PropType<User>,
  items: { type: Array as PropType<User[]> },
});
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_typed_object_prop.rs#L63) · [全部规则](all.md)

### `script/require-typed-ref`

要求无初始值、null 或 undefined 初始化的 ref() 具有显式类型参数

[错误示例](#script-require-typed-ref-bad) · [正确示例](#script-require-typed-ref-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-typed-ref": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-typed-ref-bad"></span>

**错误示例**

导入的 `ref` 调用既无类型参数，也无有效初始值：无参数、`null` 和 `undefined` 都无法推断未来值的预期类型。

```vue annotate="remove:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref()           // Ref<undefined>
const b = ref(null)       // Ref<null>
const c = ref(undefined)  // Ref<undefined>
</script>
```

<span id="script-require-typed-ref-good"></span>

**正确示例**

显式类型参数描述字符串和可空 User refs。`ref(0)` 已有具体数字初始值，可以依靠推断。

```vue annotate="add:4,5,6"
<script setup lang="ts">
import { ref } from 'vue'

const a = ref<string>()
const b = ref<User | null>(null)
const c = ref(0)          // inferred Ref<number>
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/require_typed_ref.rs#L55) · [全部规则](all.md)

### `script/require-valid-default-prop`

要求 prop 默认值符合声明类型

[错误示例](#script-require-valid-default-prop-bad) · [正确示例](#script-require-valid-default-prop-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/require-valid-default-prop": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-require-valid-default-prop-bad"></span>

**错误示例**

Number 和 Boolean props 使用了不匹配的标量默认值，Array 和 Object props 使用共享字面量而不是工厂。

```vue annotate="remove:4,5,6,7"
<script lang="ts">
export default {
props: {
count: { type: Number, default: '0' },     // string default for Number
enabled: { type: Boolean, default: 1 },     // non-boolean default for Boolean
items: { type: Array, default: [] },        // literal must be a factory
config: { type: Object, default: {} }       // literal must be a factory
}
}
</script>
```

<span id="script-require-valid-default-prop-good"></span>

**正确示例**

标量默认值改为 `0` 和 `false`；数组和对象默认值改为返回新值的函数。`[String, Number]` 示例接受字符串默认值，因为它匹配一个声明类型。

```vue annotate="add:4,5,6,7,8"
<script lang="ts">
export default {
props: {
count: { type: Number, default: 0 },
enabled: { type: Boolean, default: false },
items: { type: Array, default: () => [] },
config: { type: Object, default: () => ({}) },
label: { type: [String, Number], default: '' }
}
}
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/require_valid_default_prop.rs#L68) · [全部规则](all.md)

### `script/return-in-computed-property`

要求每个计算 getter 返回值

[错误示例](#script-return-in-computed-property-bad) · [正确示例](#script-return-in-computed-property-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-computed-property": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-computed-property-bad"></span>

**错误示例**

块函数体计算 getter 求值 `1 + 2` 却从不返回，使计算值为 undefined。

```vue annotate="remove:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { 1 + 2; });
</script>
```

<span id="script-return-in-computed-property-good"></span>

**正确示例**

`return 1 + 2` 将表达式转为 getter 的返回值。规则查找 getter 自身返回值的 return，而非仅有表达式语句。

```vue annotate="add:3"
<script setup lang="ts">
import { computed } from "vue";
const total = computed(() => { return 1 + 2; });
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/return_in_computed_property.rs#L31) · [全部规则](all.md)

### `script/return-in-emits-validator`

要求每个 Options API emits 校验函数返回值

[错误示例](#script-return-in-emits-validator-bad) · [正确示例](#script-return-in-emits-validator-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

为当前支持的 SFC 筛选使用块函数体箭头函数。底层校验器也处理方法简写，但当前 SFC 前置筛选无法可靠分派该形式。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/return-in-emits-validator": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-return-in-emits-validator-bad"></span>

**错误示例**

`submit` 校验函数记录载荷，却不返回校验结果，因此块函数体产生 undefined。

```vue annotate="remove:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { console.log(payload); } } };
</script>
```

<span id="script-return-in-emits-validator-good"></span>

**正确示例**

`return payload != null` 为提交载荷提供布尔校验结果，避免结束时没有返回值。

```vue annotate="add:2"
<script lang="ts">
export default { emits: { submit: (payload: unknown) => { return payload != null; } } };
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/props_emits/return_in_emits_validator.rs#L59) · [全部规则](all.md)

### `script/valid-define-emits`

要求 defineEmits() 用法有效（不同时提供类型和运行时参数、不引用局部变量、只调用一次）

[错误示例](#script-valid-define-emits-bad) · [正确示例](#script-valid-define-emits-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-emits": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-emits-bad"></span>

**错误示例**

同一 `defineEmits` 调用同时提供类型参数和运行时数组 `["save"]`，混合两种互斥声明。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits<{ save: [] }>(["save"]);
</script>
```

<span id="script-valid-define-emits-good"></span>

**正确示例**

移除运行时参数，只保留一个针对 `save` 的类型事件声明。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_emits.rs#L45) · [全部规则](all.md)

### `script/valid-define-options`

要求 defineOptions() 用法有效（单个对象参数、不包含 props/emits/expose/slots）

[错误示例](#script-valid-define-options-bad) · [正确示例](#script-valid-define-options-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-options": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-options-bad"></span>

**错误示例**

第一个调用将专用的 `props` 声明放入 `defineOptions`；后续调用又重复宏，并包含非对象参数。这些展示了禁止的形式和重复调用限制。

```vue annotate="remove:2,3,4,5"
<script setup lang="ts">
defineOptions({ props: ['foo'] })   // use defineProps instead
defineOptions({ name: 'Foo' })
defineOptions({ name: 'Bar' })      // duplicate call
defineOptions('Foo')                // not an object literal
</script>
```

<span id="script-valid-define-options-good"></span>

**正确示例**

一个 `defineOptions` 调用接收只包含受支持普通选项 `name` 和 `inheritAttrs` 的对象。

```vue annotate="add:2"
<script setup lang="ts">
defineOptions({ name: 'Foo', inheritAttrs: false })
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_options.rs#L41) · [全部规则](all.md)

### `script/valid-define-props`

要求 defineProps() 用法有效（只调用一次、不同时提供类型和运行时参数、不引用局部变量）

[错误示例](#script-valid-define-props-bad) · [正确示例](#script-valid-define-props-good)

默认严重程度: `error`  
预设: `essential`, `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-define-props": "error"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-define-props-bad"></span>

**错误示例**

同一 `defineProps` 调用同时提供类型参数 `{ title: string }` 和运行时参数 `{ title: String }`，编译器不允许二者组合。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps<{ title: string }>({ title: String });
</script>
```

<span id="script-valid-define-props-good"></span>

**正确示例**

移除运行时对象，只保留一个针对 `title` 的类型声明，不再组合两种声明形式。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_define_props.rs#L44) · [全部规则](all.md)

### `script/valid-next-tick`

要求 nextTick() 调用结果被等待、链式处理，或提供回调

[错误示例](#script-valid-next-tick-bad) · [正确示例](#script-valid-next-tick-good)

默认严重程度: `warning`  
预设: `happy-path`, `ecosystem`, `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 中的 JS/TS 脚本；示例展示对应的 Options API 或 script setup 形式  
选项: 没有规则专属选项。可以配置严重程度和预设。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "script/valid-next-tick": "warn"
      }
    },
  },
});
```

```sh
vp run lint
```

<span id="script-valid-next-tick-bad"></span>

**错误示例**

导入的 `nextTick()` 是无回调的独立表达式，其返回 Promise 被忽略，没有操作等待 DOM 刷新。

```vue annotate="remove:3"
<script setup lang="ts">
import { nextTick } from "vue";
nextTick();
</script>
```

<span id="script-valid-next-tick-good"></span>

**正确示例**

`await nextTick()` 使用该 Promise，显式等待下一次 DOM 更新，再继续执行后续 setup 代码。

```vue annotate="add:3"
<script setup lang="ts">
import { nextTick } from "vue";
await nextTick();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/script/valid_next_tick.rs#L55) · [全部规则](all.md)

### `type/no-floating-promises`

禁止悬空（未处理）的 Promise

[错误示例](#type-no-floating-promises-bad) · [正确示例](#type-no-floating-promises-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-floating-promises": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-floating-promises-bad"></span>

**错误示例**

异步 `save` 函数返回 Promise，但独立的 `save()` 调用既不等待也不返回它，也未显式标记有意丢弃。

```vue annotate="remove:3"
<script setup lang="ts">
async function save(): Promise<void> {}
save();
</script>
```

<span id="type-no-floating-promises-good"></span>

**正确示例**

`void save()` 显式标记此规则接受的发出后不等待意图。这是显式丢弃标记，不是拒绝处理器。

```vue annotate="add:3"
<script setup lang="ts">
async function save(): Promise<void> {}
void save();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_floating_promises.rs#L13) · [全部规则](all.md)

### `type/no-reactivity-loss`

禁止赋值和调用中对响应式值取普通快照

[错误示例](#type-no-reactivity-loss-bad) · [正确示例](#type-no-reactivity-loss-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-reactivity-loss": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-reactivity-loss-bad"></span>

**错误示例**

`const count = state.count` 对响应式属性取普通数字快照，因此之后 `state.count` 的更新不会反映到该绑定。

```vue annotate="remove:2,4"
<script setup lang="ts">
import { reactive } from "vue";
const state = reactive({ count: 0 });
const count = state.count;
</script>
```

<span id="type-no-reactivity-loss-good"></span>

**正确示例**

`toRef(state, "count")` 使 `count` 继续关联原响应式属性，不复制其当前原始类型值。

```vue annotate="add:2,4"
<script setup lang="ts">
import { reactive, toRef } from "vue";
const state = reactive({ count: 0 });
const count = toRef(state, "count");
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_reactivity_loss.rs#L12) · [全部规则](all.md)

### `type/no-unsafe-template-binding`

禁止解析为不安全类型的模板绑定

[错误示例](#type-no-unsafe-template-binding-bad) · [正确示例](#type-no-unsafe-template-binding-good)

默认严重程度: `warning`  
预设: `nuxt`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/no-unsafe-template-binding": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-no-unsafe-template-binding-bad"></span>

**错误示例**

插值的 `value` 被显式注解为 `any`，检查器无法为模板绑定提供安全的具体类型。

```vue annotate="remove:2"
<script setup lang="ts">
const value: any = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

<span id="type-no-unsafe-template-binding-good"></span>

**正确示例**

将注解改为 `string`，为同一插值提供可检查的具体类型，而不改变渲染值。

```vue annotate="add:2"
<script setup lang="ts">
const value: string = "Hello";
</script>
<template><p>{{ value }}</p></template>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/opinionated/type_aware/no_unsafe_template_binding.rs#L12) · [全部规则](all.md)

### `type/require-typed-emits`

要求 defineEmits 具有类型定义

[错误示例](#type-require-typed-emits-bad) · [正确示例](#type-require-typed-emits-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-emits": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-emits-bad"></span>

**错误示例**

仅数组形式的 `defineEmits(["save"])` 声明事件名称，却没有类型化载荷约定。

```vue annotate="remove:2"
<script setup lang="ts">
defineEmits(["save"]);
</script>
```

<span id="type-require-typed-emits-good"></span>

**正确示例**

`defineEmits<{ save: [] }>()` 以空载荷元组声明带类型的 `save` 事件，明确表示不接收载荷参数。

```vue annotate="add:2"
<script setup lang="ts">
defineEmits<{ save: [] }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_emits.rs#L54) · [全部规则](all.md)

### `type/require-typed-props`

要求 defineProps 具有类型定义

[错误示例](#type-require-typed-props-bad) · [正确示例](#type-require-typed-props-good)

默认严重程度: `warning`  
预设: `happy-path`, `nuxt`, `ecosystem`, `opinionated`  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 没有规则专属选项。可以配置严重程度和预设。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/require-typed-props": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-require-typed-props-bad"></span>

**错误示例**

仅数组形式的 `defineProps(["title"])` 只按名称声明 `title`，没有提供类型。

```vue annotate="remove:2"
<script setup lang="ts">
defineProps(["title"]);
</script>
```

<span id="type-require-typed-props-good"></span>

**正确示例**

`defineProps<{ title: string }>()` 为 `title` 提供显式字符串类型，替代仅名称的运行时声明。

```vue annotate="add:2"
<script setup lang="ts">
defineProps<{ title: string }>();
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/require_typed_props.rs#L57) · [全部规则](all.md)

### `type/strict-boolean-expressions`

要求脚本和模板条件使用安全的布尔表达式

[错误示例](#type-strict-boolean-expressions-bad) · [正确示例](#type-strict-boolean-expressions-good)

默认严重程度: `warning`  
预设: _none_  
自动修复: 无；请检查建议的修改  
适用范围: Vue SFC 脚本和模板中下方所示结构的类型信息  
选项: 参见[类型化选项和默认值](/rules/options.md)。

显式启用 typeAware 和此规则。默认不允许可空数字，允许非空数字。

类型感知检查使用原生 Corsa 运行时及 TypeScript 项目。仅设置 `typeAware` 不会启用需显式选择的规则。

**配置（Vite+）**

```ts
import { defineConfig } from "@vizejs/vite-plugin/vite-plus";

export default defineConfig({
  lint: {
    vize: {
      "preset": "incremental",
      "rules": {
        "type/strict-boolean-expressions": "warn"
      },
      "typeAware": true
    },
  },
});
```

```sh
vp run lint
```

<span id="type-strict-boolean-expressions-bad"></span>

**错误示例**

`if (count)` 依赖可空数字绑定的真值性，而非显式布尔检查，也将零与缺失混为一谈。

```vue annotate="remove:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count) console.log(count);
</script>
```

<span id="type-strict-boolean-expressions-good"></span>

**正确示例**

`count !== undefined && count > 0` 分别检查存在性和正值，缩窄可选值后产生显式布尔条件。

```vue annotate="add:3"
<script setup lang="ts">
const count: number | undefined = undefined;
if (count !== undefined && count > 0) console.log(count);
</script>
```

正确示例在上述配置下不会触发这条规则的诊断；其他规则仍可能报告诊断。

[实现](https://github.com/ubugeeei-prod/vize/blob/main/crates/vize_patina/src/rules/type_aware/strict_boolean_expressions.rs#L7) · [全部规则](all.md)
