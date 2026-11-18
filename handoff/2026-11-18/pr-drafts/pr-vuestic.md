Fixes a Real Project Matrix false positive in vuestic-admin. `vize check` reported TS7006 "Parameter 'v' implicitly has an 'any' type" at `src/pages/auth/Signup.vue:38:12` and `:39:12`. vue-tsc reports neither.

**What was wrong.** A component with a listener also gets an inference-only copy of its props, the emit-props literal. That copy gives callbacks no contextual type, and each entry is guarded by a generated `@ts-ignore` that only covers the entry's first line. A callback inside a multi-line array or object value, like the multi-line `:rules="[ (v) => ... ]"` on that `VaInput`, starts below the ignored line, so its parameter leaked TS7006. The canonical props call types the same parameter correctly. The one-line `:rules` on line 10 of the same file stays clean. The `\'` escape was not the trigger. The content mapper reported the same false positives, in places twice.

**Fix.** Callback parameters that start below the first line of a multi-line non-callback value now move onto generated `@ts-ignore` lines inside their parentheses. Nested callback parameters in the other inference copies already work this way. The inserted directives are synthetic ranges, so they stay unmapped. This applies only to that mapped inference copy, and not where template-binding access rewrites the value. Every other projection is byte-identical.

**Tests.**
- `content_mapper_tsgo_cli::multiline_library_prop_callbacks`: a declarations-only library component (`DefineComponent` with `rules`, an object-callback prop and emits) with a listener. Multi-line arrays, the `\'` escape, comments, nested callbacks and an object value are clean under both the mapper and `vize check`. Inside the same shapes, the genuine TS2339 (`v.missingMethod()`) and TS2322 (a wrong `(v: number)` annotation) still report once. On main this project reports 6 to 9 false TS7006s.
- `nested_callbacks::tests::value_callbacks_below_the_first_line_get_directives_with_ranges` covers the rewrite and its synthetic ranges.
- The fixture is registered in the typechecker differential manifest.
- The producer custody chain gains a `multiline-value-callbacks` transition for `generic_inference.rs`. The emit-props gate tooling suites match main.

Verified against the pinned vuestic-admin with installed deps: `Signup.vue` is clean.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
