# Script setup duplicate keys (#6881)

Issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881). Supersedes the unmerged proposal in [#8763](https://github.com/ubugeeei-prod/vize/pull/8763), whose implementation this change carries without its source-custody packets.

The v0.447.0 Real Project Matrix (run 38205026873 at `ef34d1a0409fc711a6c5cefcf0708b0cce78f502`) recorded 88 `script/no-dupe-keys` false negatives and no false positives across 51 Vue files from eight projects. The reference is `vue/no-dupe-keys` from eslint-plugin-vue 10.9.2 with vue-eslint-parser 10.4.1.

The rule and its dispatch prefilter looked at Options API objects only. The rule now also checks compiler-owned `<script setup>` props against the resolved value bindings of the setup block and of the sibling normal script:

- A binding whose initializer derives from the same props declaration (for example a destructure of the `defineProps` result or a value computed from the props object) is not a collision; a renamed destructure target is not one either.
- Type-only names and bindings owned by an unrelated lexical scope never establish a collision.
- Existing Options API diagnostics, their order and their spans are unchanged.
- The dispatcher reuses the parses it already has; no additional parse or pipeline stage is added.

Five `npm/ui` date-time components declared an `id` prop and a setup `id` binding. They now alias the resolved value as `resolvedId`; public props, exposed values and rendered attributes are unchanged.

## Regression coverage

`crates/vize_patina/tests/no_dupe_keys_447.rs` runs the real linter over the 51 original files (`tests/_fixtures/differential/linter/no-dupe-keys-447/original-sources/`) and asserts every diagnostic field for all 88 findings, 39 authored positive/negative boundary controls, eight inheritance and ownership neighbors, and configured severity/disabled behavior. Unit tests cover the prefilter and dispatch order.

## Known gap

Generic argument specialization is unfinished. For `type Props<T> = T` the official compilers can resolve an unrelated root or sibling `T` and emit a runtime prop that the reference rule does not report; this change keeps reporting that genuine runtime collision. When no such `T` exists, a prop introduced only through the generic argument is still missed. Track it on #6881.

The rule-contract `domain` text for `script/no-dupe-keys` (shown by `vize explain`) still names Options API objects only. The whole-file custody pins on `crates/vize/src/commands/explain/snapshots/` reject any unrelated snapshot edit, so the wording is updated together with retiring those pins rather than in this fix.
