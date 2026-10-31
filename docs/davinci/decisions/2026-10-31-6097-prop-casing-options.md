# #6097 declared prop casing option

`vue/prop-name-casing` already implements camelCase and snake_case declaration
checks. Expose that existing choice as the typed public option
`{ casing: "camelCase" | "snake_case" }`, with camelCase as the default. Keep the
declaration predicate and authored ranges unchanged; add help for the selected
snake_case policy without changing existing camelCase text.

The configurable-rule audit now records 17 mapped rules with typed options and 43
without, while retaining the same 112 configurable upstream rules and pinned
eslint-plugin-vue 10.9.2 executable provider. This is one option slice, not
completion of #6097 or support for upstream `ignoreProps`.

The independently authored `prop-name-casing-options-6097` input pins five full
provider diagnostic vectors: default, explicit camelCase, explicit snake_case,
and two clean template-only controls. The provider has no safe rename fixer, and
its complete verify-and-fix results retain every input byte. The public CLI tests
consume those exact registered files and pin complete messages, help, ranges,
statuses and unchanged inputs, including `--fix`, scoped replacement, empty
policy reset, disabled/unselected rules and invalid configuration.

The input-only differential registration is source-inspected and retains unknown
native/legacy acceptance. It does not recapture or modify any historical oracle.
Acceptance requires the pinned executable provider replay, public Rust/CLI config
tests and repository Pkl 0.27.2 generation on Actions. The earlier local Pkl 0.31
preview does not substitute for that pinned generation check. No local native
build or native performance result is claimed.
