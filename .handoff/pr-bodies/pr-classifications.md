## Summary

This PR changes matrix tooling only: Patina is untouched. It bundles the lint-matrix classification work of the real-project lane into one PR, to save merge-queue rounds. Every classification is narrow, follows the standing rule (Patina is right and the difference comes from a limitation of the reference) and has tests for both sides. Each item is its own commit, so any one can be reverted alone.

Together with the other lane PRs, the local all-branch measurement on the 57 projects shared with run 44093557816 goes from 5230 FP / 3645 FN to 5 FP / 2 FN.

## Part 1: documented extensions and reference misreadings

This PR classifies findings that the Real Project Matrix (#12336) still counts as divergence. In each case Patina is correct and the difference comes from the reference: a documented Patina extension, a reading eslint-plugin-vue gets wrong, or how it represents a location. It changes tooling only.

First, three mapped rules intentionally differ from eslint-plugin-vue, and each rule's docs record the difference. Each finding is classified with the reason and the issue that settled it.

| Rule | Patina's documented behavior | Upstream | Classified |
| --- | --- | --- | --- |
| `vue/next-tick-style` | also checks `<script setup>` (#12450, kept in #12670) | `defineVueVisitor` only visits component option objects | Patina findings inside `<script setup>` |
| `script/valid-next-tick` | applies in `<script setup>` as well as component options (#12794, rule docs) | same | Patina findings inside `<script setup>` |
| `script/custom-event-name-casing` | leaves the model prop name after `update:` unchecked (#9879, restored in #12592 for #12587) | checks every `:`-separated segment | upstream `Custom event name 'update:<prop>'` findings |

These findings stay counted:
- findings in component options, for both nextTick rules;
- any custom event name other than `update:<non-empty>`.

Every script/valid-next-tick false positive in run 44093557816 is inside `<script setup>`: 12 across dbx, frappe-builder, inertia, misskey, nuxt-ui, scalar, splitpanes, tiny-rdm and vuetorrent. I checked each one.

### eslint-plugin-vue misreads in vue/no-duplicate-class-names

Two upstream readings are wrong, and Patina gets both right. Each classification reads only the start tag of the finding:

- **Identifier object keys.** `:class="{ hidden: wide }"` renders `hidden`, but upstream counts only string-literal keys. A Patina `html/no-duplicate-class` finding is classified when every name it lists appears as an identifier key and at most once anywhere else (the static value, string literals or quasi tokens). Repeats through string keys or within a static value stay counted.
- **Glued template fragments.** Upstream splits quasis on whitespace, so `-600` in `` `hover:text-${color}-600` `` counts as a class. An upstream finding is classified when every name it lists is glued to a `${}` interpolation and appears whole at most once. A whole token bounded by whitespace stays counted.

This covers the six residual findings left by the html/no-duplicate-class location PR: npmx AppHeader, hoppscotch teams/index and tab/Primary and tab/Secondary, vue-vben-admin tabs-chrome, and element timeline/item.

### Self-references by file name

Vue's template compiler names an SFC after its file, as `capitalize(camelize(base))`, and resolves a tag with that name to the component itself, so `<x-icon>` inside `XIcon.vue` needs no registration. eslint-plugin-vue's `vue/no-undef-components` does not read the file name. All 68 of its `require-component-registration` false negatives in run 44093557816 are such tags: vux 39, epic-spinners 20, primevue 3, primevue-showcase 3, vuestic-ui 2 and cube-ui 1. Those findings are now classified; a tag that differs from the file name stays counted.

### require-typed-ref under a type assertion

Patina keeps its stricter stance (#12069, pinned test): `ref(null) as unknown as Ref<T>` is an unchecked cast, so it still needs a type argument. eslint-plugin-vue counts any `as`/`satisfies` assertion as typed. The 29 such findings (frappe-builder 23, reka-ui 6) are classified when the reported call, through grouping parentheses only, is the operand of `as` or `satisfies`. A call passed to another function, such as `cast(ref(null)) as Ref<T>`, stays counted.

### SVG elements in v-if-else-key

Vue's compiler treats every tag in its SVG list as native wherever it appears. vue-eslint-parser places a `<path>` outside an `<svg>` ancestor, for example inside a component's slot, in the HTML namespace, so `vue/v-if-else-key` reads it as a custom component and asks for a key. That accounts for all 10 findings: npmx 5, reka-ui 2, vue-charts 2, vue-flow 1. Each is classified when the tag is in Vue's SVG list.

### Payload calls before defineEmits

`vue/prefer-single-event-payload` learns the `defineEmits` binding only once its visitor reaches the macro, so it skips script calls written earlier, for example inside a function declared above `const emit = defineEmits(...)` (alexandrie Toolbar.vue). Patina resolves the binding first. Such script findings now get the reason already used for `script/custom-event-name-casing`. Template calls, which upstream visits after the script, stay counted.

### Emits eslint-plugin-vue cannot see

`vue/no-unused-emit-declarations` sees the `defineEmits` binding only as a bare identifier it has already reached. It therefore misses emits through a type assertion (`(emit as any)('apply', payload)`, `useVModel(props, 'modelValue', emits as any)`) and calls written above the macro, and reported those events as unused: airi 2, reka-ui 2, hoppscotch 1. Patina counts them, so these upstream findings are classified. The same applies to a `bind`, `call` or `apply` on the binding (n8n `DataTableTable.vue`, `emit.bind(null, 'toggleSave')`).

### Parse error points

`vue/no-parsing-error` reports a zero-width point, while Patina's `parser/template` ranges the offending token, so each shared error split into a false positive and a false negative. Such a pair is now recorded as a rule location divergence when the point lies on the same line inside Patina's range and the error code names Patina's message. For example, `missing-whitespace-between-attributes` pairs with "Missing whitespace between attributes", and an expression error pairs with Patina's expression error. That covers 10 pairs: vux 6, npmx 2, vue-manage-system 1 and vue2-elm 1. A different code, position or severity stays apart; vue2-elm's duplicate attribute is a warning in Patina, so it is not paired.

### Props inherited through extends

eslint-plugin-vue reads the `defineProps<T>()` type and the unions, intersections and aliases T spells, but it never follows an interface `extends` clause. Patina does. With eslint-plugin-vue 10.12.0, `interface P extends Base {}` yields no `vue/require-prop-comment` for `Base` members, whether P is used alone, in a union or in an intersection.

- `vue/require-prop-comment` on a member of a same-file interface is classified when that interface is mentioned only in its declaration and in the `extends` clauses of interfaces the props type reaches. Examples: directus `slider.vue` `BaseProps` under `defineProps<NumberProps | LegacyNumericStringProps>()` (6), and nuxt-ui `Table.vue` (3).
- `vue/no-props-shadow` and `vue/no-template-shadow` findings are classified when the shadowed prop is declared only in such an inherited interface. In that case no type eslint-plugin-vue reads names the prop, the heritage path quotes it in no `Omit` and misses no `Pick`, and, for template shadows, no script binding or import declares the name. Examples: nuxt-ui `Link.vue` `to`/`rel`/`href` (5) and reka-ui `MenuContentImpl` `trapFocus`/`disableOutsidePointerEvents` (2). These appear once props are read from the sibling `<script>`.

### Asserted method handlers

Vue's compiler unwraps TypeScript `as`/`satisfies`/`!` when it decides that a v-on value is a method handler. I checked this with compiler-dom 3.5.45: `@keydown="(handleKeyDown as any)"` compiles to `onKeydown: (_ctx.handleKeyDown as any)`, with no arrow wrapper. `vue/v-on-handler-style` in eslint-plugin-vue reads that value as an inline handler (reka-ui `SelectContentImpl.vue`, 1). A member path under an assertion is classified.

### Inline intersection props beside a defaults spread

Since #12069, `script/require-default-prop` keeps checking a prop the file appends in an inline intersection member (`type Props = Imported & { clearValue?: T }`) when `withDefaults` spreads imported defaults, because the spread covers the imported contract. eslint-plugin-vue skips every prop once the defaults object holds a spread. The classifier mirrors Patina's own boundary: the member must be in a `type X = A & { ... }` operand, and the `withDefaults` object must spread a value (vuestic-ui `VaFormField.vue`, 1).

### Tests

`tests/tooling/lint-documented-rule-extensions.test.ts` covers both sides of each rule:
- next-tick-style and valid-next-tick findings in `<script setup>` are classified; findings in component options are not;
- `update:selected-date` is classified; `selected-date` and `close-dialog:done` are not.
- identifier-key and array-holder findings are classified; a string-literal key and a static repeat are not.
- glued `-600` and `node--` fragments are classified; a whitespace-bounded `px-2` is not.
- `ref(null) as unknown as Ref<…>` and `(ref()) satisfies …` are classified; a plain `ref(null)` and `cast(ref(null)) as …` are not.
- `path`, `g`, `line`, `polyline` and `text` are classified; `MyIcon`, `my-path` and `Path` are not.
- a payload call inside a function above `defineEmits` is classified; calls after the macro and in the template are not.
- an assertion-wrapped emit, an emit above the macro and an `emit.bind` hand-off are classified; an event with no emit and another binding's `bind` are not.
- `lint-inherited-prop-members.test.ts`: base members and heritage-only shadowed props are classified, including through an alias; own members, a base also read directly or bound elsewhere, an `Omit` of the name, a script binding and an import are not.
- an asserted member path handler is classified; a call, a function-typed assertion and a statement are not.
- an inline-intersection prop beside a spread is classified; without the spread, or inline in `defineProps<…>`, it is not.
- `lint-parse-error-points.test.ts`: four code-matched points pair; another code, position or severity does not.
- `lint-divergence-component-registration-intentional.test.ts`: `x-icon` in `XIcon.vue`, `AtomSpinner` and `atom-spinner` are classified; `x-icon` in `Icon.vue` or `XIcons.vue`, or with no file, is not.

### Local measurement

`lint-divergence-report --all-mapped-rules`. For each project: FP/FN remaining for the three rules, and the number of findings this PR classifies.

| Project | Remaining | next-tick-style | valid-next-tick | custom-event-name-casing |
| --- | --- | --- | --- | --- |
| dbx | 0 | 154 | 3 | 2 |
| misskey | 0 | 52 | 1 | 0 |
| scalar | 0 | 16 | 1 | 0 |
| splitpanes | 0 | 3 | 1 | 0 |
| tiny-rdm | 0 | 8 | 2 | 0 |
| vue-cal-v4 | 1 FP / 1 FN | 0 | 0 | 2 |
| vue-vben-admin | 0 | 7 | 0 | 1 |

The vue-cal-v4 pair is a location difference on a component-options `this.$nextTick`. The next-tick-style identifier-range fix, opened separately, handles it.

## Part 2: findings that differ only through an imported contract

The eslint-plugin-vue baseline in the Real Project Matrix (#12336) runs without type-aware parser services. A prop, event or slot that a macro receives from another module is therefore unknown to it, while Patina resolves the module. This PR classifies the findings that differ only for that reason, following Option A from the lane review:

| Rule | Side | Classified when |
| --- | --- | --- |
| `script/require-default-prop`, `script/no-required-prop-with-default`, `vue/prop-name-casing`, `script/no-boolean-default` | Patina FP | the range is in the `defineProps<T>` type argument or its defaults, T reaches an import binding, and no same-file declaration names the prop; a PascalCase name that the argument references as a type stays counted |
| `script/no-dupe-keys`, `vue/no-template-shadow` | Patina FP | the colliding binding, `defineModel` name or template variable reuses a prop that only an imported type or an imported runtime props object (`defineProps(affixProps)`) declares; the statement holding the finding is the only same-file declaration of the name |
| `vue/no-shadow-native-events`, `script/no-unused-emit-declarations` | Patina FP | the finding is on a `defineEmits<T>` type argument that reaches an import, and no same-file type it reaches spells the event |
| `vue/require-explicit-slots` | upstream FN | the `<slot>` has a static or absent name (a bound literal such as `:name="`month-overlay-value`"` counts as static), the only `defineSlots<T>` reaches an import, and no same-file type it reaches spells the slot |

"Reaches" follows local aliases and interface heritage through the same-file text. For every rule, a "same-file type" is one the macro's type argument reaches. Slot, emit and context interfaces often repeat a prop name, such as a slot's `open` or reka-ui `DropdownMenuRootContext`'s `modal` and `dir`, so those are not counted. If the closure is cut at its visit cap, every type counts. A destructure that reads the props object back (`const { tab, ...rest } = props`) declares nothing new, so it is not counted either. Bracket matching skips comments, because an apostrophe in a comment inside a `withDefaults` object (`Vue's`) used to hide the object. Every case the file declares itself stays counted.

### Evidence

Run 44093557816 contains, among others, these findings, all checked by hand:
- `script/no-dupe-keys`: vben-layout `sidebarCollapse = defineModel('sidebarCollapse')` against the imported `VbenLayoutProps`; reka-ui `dir = useDirection(propDir)` against `Pick<RovingFocusGroupProps, 'dir'>`; element-plus `target = shallowRef()` against `AffixProps`.
- `vue/no-template-shadow`: `v-for="(id, index) in 5"` against `PinInputRootProps`; nuxt-ui `v-slot="{ open }"` against `Pick<CollapsibleRootProps, 'open'>`.
- `vue/no-shadow-native-events`: `defineEmits<MiniMapNodeEmits>()` from `./types`, plus 46 more across reka-ui, directus, dbx, vben, vaul-vue and vue-datepicker.
- `vue/require-explicit-slots`: scalar `defineSlots<ComboboxSlots<O, G>>()`, vue-datepicker `defineSlots<ActionRowSlots>()`, and others; 115 findings in total.

Local measurement with #13286's isolation in place:

On the local all-branch measurement (all lane branches on main, with #13286's isolation), these imported-contract classifications cover:

| Rule | Classified | Projects |
| --- | --- | --- |
| `vue/require-explicit-slots` | 266 | 7 |
| `vue/no-shadow-native-events` | 49 | 7 |
| `script/no-dupe-keys` | 31 | 6 |
| `script/no-required-prop-with-default` | 7 | 3 |
| `vue/no-template-shadow` | 4 | 3 |
| `vue/prop-name-casing` | 4 | 2 |
| `script/no-unused-emit-declarations` | 1 | 1 |

The `require-default-prop` / `no-boolean-default` imported-contract reason that main already had now covers 816 and 365 findings. Unreached context interfaces no longer block it.

### Tests

- `lint-imported-prop-contract.test.ts` and `lint-imported-prop-collision.test.ts`: classified are imported types, imported runtime objects, renamed destructure keys, slot or context interfaces that repeat a prop name, defaults behind a comment, and a `= props` read-back destructure. Not classified are same-file members (including one reached through `extends`), runtime keys, second declarations, destructures of other values or props members, and local-only props.
- `lint-imported-macro-contract.test.ts`: imported emits and slots types are classified, including a slot named by a bound literal; same-file members, call signatures, local interfaces and slots with a bound non-literal `:name` are not.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
