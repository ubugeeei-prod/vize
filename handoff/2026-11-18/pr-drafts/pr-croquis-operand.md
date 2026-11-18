## Summary

Croquis' type-member scanner joins a member with the next line when that line starts with `|` or `&` (#12592's `inputmode?:\n | "text"` case). A line that *ends* with the operator was still flushed at the newline. So in scalar `ScalarListbox.vue` (Real Project Matrix, #12336):

```ts
defineProps<
  {
    options: Option[]
    id?: string
  } & (SingleSelectListboxProps | MultipleSelectListboxProps) &
    ScalarFloatingOptions
>()
```

the operand `ScalarFloatingOptions` was read as an unannotated member, which made it a prop. `vue/prop-name-casing` reported `Prop "ScalarFloatingOptions" is not in camelCase`, and `vue/no-unused-properties` reported it as unused. eslint-plugin-vue reports neither.

A newline after a trailing `|` or `&` now continues the member as well. As a side effect, an interface member written as `size?: Small &\n  Large` keeps its full type instead of splitting into `size: Small &` plus a bogus `Large` prop.

## Tests

- `types::tests::extract_properties_keeps_a_trailing_operator_operand_off_the_props`
- `prop_name_casing::tests::an_intersection_operand_on_its_own_line_is_not_a_prop`
- Differential case `linter/intersection-operand-line-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
