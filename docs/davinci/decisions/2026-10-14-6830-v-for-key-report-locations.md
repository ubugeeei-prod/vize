# `vue/require-v-for-key` report locations (#6830)

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830). Narrows [the iteration key ownership record](./2026-10-13-6830-v-for-key-rule-ownership.md) by replacing its "historical any-keyed-child policy".

The v0.452.0 Real Project Matrix (run 38515301568) recorded 46 false positives, 72 false negatives and 589 location mismatches for `vue/require-v-for-key` against eslint-plugin-vue 10.9.2. All three come from two differences:

- Vize reported the `v-for` attribute. The reference reports the repeated element's opening tag (`<li v-for="…">`).
- For an unkeyed `<template v-for>`, Vize reported the template once and accepted it if any child had a `:key`. The reference forwards the check to each child element and reports each unkeyed native child at its opening tag. A nested `<template>` / `<slot>` forwards again.

Vize now does the same. A forwarded child counts as keyed only with a bound `:key`. The root's static `key="…"` and `v-bind="{ key }"` exemptions from #8281 stay at the root, and custom components stay owned by `vue/valid-v-for`. A child with its own `v-for` is checked by that loop. JSX lists keep their current ranges.

In the 21 slot-fallback cases, 18 now match the recorded eslint-plugin-vue packets. The other 3 are the deliberate root exemptions and one empty-slot case that the test does not compare. The "any-key policy limit" cases, which used to be clean, now report the unkeyed sibling as the reference does. Pug templates report at the element's tag name.
