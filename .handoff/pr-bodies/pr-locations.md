## Summary

These five Patina fixes change only where a finding is reported: what each rule finds stays the same. eslint-plugin-vue found the same problems but at a different range, so in the Real Project Matrix (#12336) each finding split into a false positive and a false negative. Each fix is a separate commit with its own tests and differential case, so any one can be reverted alone.

### 1. `vapor/prefer-static-class` reports the static class nodes (about 300 FP / 150 FN)

It reported the whole `:class` binding for a constant and cut static fragments out of template quasis such as `callout__${type}`. It now reports each static class node the way eslint-plugin-vue does:
- a string literal;
- a whole template quasi bounded by whitespace or the literal's ends, including its `` ` ``, `${` and `}` delimiters as ESTree ranges it;
- an array item;
- an object key whose value is `true`.

A quasi that is part of a dynamic class (`badge--` in `` `badge--${variant}` ``) is skipped whole. The autofix still replaces the whole binding.

Re-pinned to match: the static-class cohort `cases.json` sha, the oracle-index snapshot shas, the manifest oracle sha and the `differential-linter-sfc-contract` pins, the battery census (+1 call, +1 skip, with accounting), the snapshots and the docs anchors. Edits and applied bytes are unchanged.

### 2. `vue/next-tick-style` reports the `nextTick` identifier (about 500 FP / 500 FN)

It now reports the `nextTick` / `$nextTick` identifier instead of the whole callee:
- `$nextTick` counts only on `this` or on a `const` alias of `this` in scope (new `this_alias.rs`);
- `Vue.nextTick` counts on any `Vue` binding;
- `?.then` consumes the Promise, like `.then`.

The docs anchor is refreshed.

### 3. Emit rules report eslint-plugin-vue's nodes

| Rule | Case | Before | Now |
| --- | --- | --- | --- |
| `vue/no-shadow-native-events` | a `defineEmits<T>()` type mixing an imported part with a member the file declares (`Omit<RootEmits, ...> & { change: [event: Event] }`) | the type argument | the file's own `change` member |
| `vue/prefer-single-event-payload` | a parenthesized payload, `emit('toggled', reaction, (count - 1))` | including the parentheses | inside them, since ESTree has no parenthesized node |

When the type world resolves a `defineEmits` type, each event that the file's own members also declare keeps that member's range and payload. nuxt-ui goes from 5 FP + 5 FN to 0, and misskey from 3 + 3 to 0.

### 4. `vue/no-restricted-v-bind` reports the directive key

It reported the whole `:v-bind="{ item: recipe }"` attribute. It now reports from the attribute start up to `=`, or up to the first whitespace for a valueless attribute, keeping modifiers as the ESTree `VDirectiveKey` does (mealie, 2). The docs anchor is refreshed.

### 5. Report columns skip a leading byte order mark

ESLint drops a leading BOM before reporting positions, but Patina's reporters counted it as column 1 of line 1. dbx `AiAssistant.vue`'s `vue/block-lang` was off by one as a result. `SourceLineIndex` now starts line 1 after the mark. `crates/vize_patina/src/output/shared.rs` is a registered report source, so its new sha256 (after #13245) is appended to `CURRENT_REPORT_SOURCES`; the three current report packets are byte-identical.

## Tests

- `prefer_static_class` rule test: a literal, a whole quasi, a fragment quasi, and array and object items.
- `next_tick_style::tests`: exact identifier ranges, `this` aliases in and out of scope, a non-`this` receiver, `Vue.nextTick` and `?.then`.
- `native_events_declared_on_a_local_member_report_that_member`, and `parenthesized_payloads_are_ranged_inside_their_parentheses` (one and two levels).
- `restricted_v_bind_reports_the_directive_key`: `:v-bind`, `:v-model` and `:v-show.camel`.
- `output::tests::byte_order_mark::json_columns_skip_a_leading_byte_order_mark`.
- Differential cases `linter/prefer-static-class-nodes-12336`, `next-tick-style-identifier-12336`, `emit-report-nodes-12336`, `no-restricted-v-bind-key-12336` and `bom-columns-12336`.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
