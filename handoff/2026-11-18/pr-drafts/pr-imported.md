## Summary

The eslint-plugin-vue baseline in the Real Project Matrix (#12336) runs without type-aware parser services. A prop, event or slot that a macro receives from another module is therefore unknown to it, while Patina resolves the module. This PR classifies the findings that differ only for that reason, following Option A from the lane review:

| Rule | Side | Classified when |
| --- | --- | --- |
| `script/require-default-prop`, `script/no-required-prop-with-default`, `vue/prop-name-casing`, `script/no-boolean-default` | Patina FP | the range is in the `defineProps<T>` type argument or its defaults, T reaches an import binding, and no same-file declaration names the prop; a PascalCase name that the argument references as a type stays counted |
| `script/no-dupe-keys`, `vue/no-template-shadow` | Patina FP | the colliding binding, `defineModel` name or template variable reuses a prop that only an imported type or an imported runtime props object (`defineProps(affixProps)`) declares; the statement holding the finding is the only same-file declaration of the name |
| `vue/no-shadow-native-events`, `script/no-unused-emit-declarations` | Patina FP | the finding is on a `defineEmits<T>` type argument that reaches an import, and no same-file type it reaches spells the event |
| `vue/require-explicit-slots` | upstream FN | the `<slot>` has a static or absent name (a bound literal such as `:name="`month-overlay-value`"` counts as static), the only `defineSlots<T>` reaches an import, and no same-file type it reaches spells the slot |

"Reaches" follows local aliases and interface heritage through the same-file text. For every rule, a "same-file type" is one the macro's type argument reaches. Slot, emit and context interfaces often repeat a prop name, such as a slot's `open` or reka-ui `DropdownMenuRootContext`'s `modal` and `dir`, so those are not counted. If the closure is cut at its visit cap, every type counts. A destructure that reads the props object back (`const { tab, ...rest } = props`) declares nothing new, so it is not counted either. Bracket matching skips comments, because an apostrophe in a comment inside a `withDefaults` object (`Vue's`) used to hide the object. Every case the file declares itself stays counted.

## Evidence

Run 44093557816 contains, among others, these findings, all checked by hand:
- `script/no-dupe-keys`: vben-layout `sidebarCollapse = defineModel('sidebarCollapse')` against the imported `VbenLayoutProps`; reka-ui `dir = useDirection(propDir)` against `Pick<RovingFocusGroupProps, 'dir'>`; element-plus `target = shallowRef()` against `AffixProps`.
- `vue/no-template-shadow`: `v-for="(id, index) in 5"` against `PinInputRootProps`; nuxt-ui `v-slot="{ open }"` against `Pick<CollapsibleRootProps, 'open'>`.
- `vue/no-shadow-native-events`: `defineEmits<MiniMapNodeEmits>()` from `./types`, plus 46 more across reka-ui, directus, dbx, vben, vaul-vue and vue-datepicker.
- `vue/require-explicit-slots`: scalar `defineSlots<ComboboxSlots<O, G>>()`, vue-datepicker `defineSlots<ActionRowSlots>()`, and others; 115 findings in total.

Local measurement with #13286's isolation in place:

MEASUREMENT

## Tests

- `lint-imported-prop-contract.test.ts` and `lint-imported-prop-collision.test.ts`: classified are imported types, imported runtime objects, renamed destructure keys, slot or context interfaces that repeat a prop name, defaults behind a comment, and a `= props` read-back destructure. Not classified are same-file members (including one reached through `extends`), runtime keys, second declarations, destructures of other values or props members, and local-only props.
- `lint-imported-macro-contract.test.ts`: imported emits and slots types are classified, including a slot named by a bound literal; same-file members, call signatures, local interfaces and slots with a bound non-literal `:name` are not.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
