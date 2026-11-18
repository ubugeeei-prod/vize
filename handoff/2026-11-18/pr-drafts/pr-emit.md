## Summary

Two emit rules found the same problems as eslint-plugin-vue but reported a different range. Each finding therefore split into a false positive and a false negative in the Real Project Matrix (#12336):

| Rule | Case | Before | Now (eslint-plugin-vue's node) |
| --- | --- | --- | --- |
| `vue/no-shadow-native-events` | a `defineEmits<T>()` type that mixes an imported part with a member this file declares (`Omit<RootEmits, ...> & { change: [event: Event] }`) | the `defineEmits` type argument, because the imported part sends resolution through the type world | the file's own `change` member |
| `vue/prefer-single-event-payload` | a parenthesized payload, as in `emit('toggled', reaction, (count - 1))` | the range including the parentheses | inside the parentheses, since ESTree has no parenthesized node |

When the type world resolves a `defineEmits` type, each event that the file's own members also declare now keeps that member's range and payload. Events that only the imported part declares are still reported on the type argument; those are classified separately as imported contracts.

## Tests

- `compatibility_sfc::tests::native_events_declared_on_a_local_member_report_that_member`
- `event_payload::tests::parenthesized_payloads_are_ranged_inside_their_parentheses`, covering one and two levels of parentheses.
- Differential case `linter/emit-report-nodes-12336`.

Local `lint-divergence-report --all-mapped-rules`: nuxt-ui `vue/no-shadow-native-events` goes from 5 FP + 5 FN to 0 (CheckboxGroup, Listbox, PinInput, RadioGroup). misskey `vue/prefer-single-event-payload` goes from 3 FP + 3 FN to 0 (MkReactionsViewer). The other projects measured (reka-ui, dbx, directus, vaul-vue, vue-datepicker, vue-flow, vue-vben-admin) show only the imported-type findings that the classification PR covers.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
