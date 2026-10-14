# Reference tests stop pinning live producers

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Eight reference tests compared live repository files byte for byte against a
recorded hash or a recorded whole body. Any edit to those files, including a
trailing newline, failed the test without any change in behaviour:

| Test                                | Live file it pinned                                                                       |
| ----------------------------------- | ----------------------------------------------------------------------------------------- |
| `reserved-suppression-reference`    | `crates/vize_patina/src/context/eslint_directive.rs`                                      |
| `vapor-keyed-fragment-expectations` | the JSX Vapor snapshots and `adversarial_snapshots.rs`                                    |
| `public-vue-imports-reference`      | the patina test, lint test and two tooling helpers                                        |
| `public-vue-imports-doc-fences`     | the source-law producer, corpus and table                                                 |
| `docs-rule-example-indentation`     | the rule sources the examples come from                                                   |
| `browser-origin-current-reference`  | `crates/vize_croquis_cf/tests/browser_diagnostic_origin.rs`                               |
| `template-emit-navigation-receipt`  | `crates/vize_canon/src/virtual_ts/scope/component_slots.rs` and the receipt script itself |
| `vite-vue-compat-control`           | the live Vite+ producers under `npm/`                                                     |

Each test keeps its archived fixtures, their hashes and every behavioural
assertion. Live files are now required to exist, and where the test proves a
specific transition (the Vapor #8625 dispatch, the browser-origin help text,
the TSX input) it checks that content directly instead of the whole file.
