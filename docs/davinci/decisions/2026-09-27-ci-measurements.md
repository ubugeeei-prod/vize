# Rust test phase measurements — #6861

Tracked in [#6861](https://github.com/ubugeeei-prod/vize/issues/6861), under
the [CI tier decision](./2026-09-27-level-restructure.md#ci-tiers).

The source-check Rust gate first builds workspace tests with
`cargo test --workspace --no-run --timings`, then executes the unchanged
`cargo test --workspace`. Both retain required tsgo, feature selection,
profile, thread counts, source selection and cache namespaces. Clippy,
fixture coverage and the strict aggregate remain required as before.

Each phase records elapsed whole seconds and the exact Cargo exit code,
then returns that code. Failed compilation skips execution. Collection or
upload cannot make failed Cargo pass; missing, invalid, skipped or cancelled
evidence is identified separately from a successful sample.

The run/attempt/tested-SHA artifact contains:

- phase JSON and a summary with each Actions outcome;
- fresh Cargo timing HTML, after removing only the previous canonical
  report before compilation, with a start marker excluding old HTML when
  compilation never ran;
- checked-out and event-head identity, active rustc/Cargo versions, and
  allowlisted event/run/attempt/command/profile/cache metadata.

The reporter rejects missing records for executed phases, invalid elapsed
times or exit statuses, status/outcome mismatches, and missing HTML after a
successful build. It never dumps environment variables or uploads historic
cached timing reports. Compiled targets and cache namespaces are unchanged.

Execution includes doctests and their residual compilation. Cargo HTML
shows compiler-unit timings; these measurements do not record CPU time/RSS
or establish a speedup, controlled cold/warm result, p50/p90 distribution,
or full T1 coverage. Record actual runner/toolchain/cache identity when
comparing subsequent completed runs.

Validate phase commands, exit statuses, fresh HTML and exact checked-out/
event-head identity from normal automatic PR and merge-group checks before
closing #6861. Do not dispatch optional repeated/full checks for this change.
Required full T1 coverage (#6865) precedes removing corresponding work from
T0; instruction-count enforcement remains unfinished under #6868.
