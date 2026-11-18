## Summary

`vue/next-tick-style` reported the whole callee (`this.$nextTick`) where eslint-plugin-vue reports the `nextTick` or `$nextTick` identifier. Every Options API finding therefore split into a false positive and a false negative in the Real Project Matrix (#12336); there were about 500 of each.

The rule now matches upstream's reading:

- it reports the `nextTick` / `$nextTick` identifier;
- `$nextTick` counts only on `this` or on a `const` alias of `this` in scope (new `this_alias.rs`);
- `Vue.nextTick` counts on any `Vue` binding;
- `?.then` consumes the Promise, like `.then`.

Findings inside `<script setup>`, which upstream does not visit (#12450, #12670), are classified separately in the matrix tooling.

## Tests

- `next_tick_style::tests`: exact identifier ranges, `this` aliases in and out of scope, a non-`this` receiver, `Vue.nextTick`, and `?.then`.
- Differential case and refreshed docs anchor.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
