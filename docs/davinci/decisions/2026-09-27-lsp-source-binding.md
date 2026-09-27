# Full tooling LSP source identity

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830) and
[#6883](https://github.com/ubugeeei-prod/vize/issues/6883).

Full merge-group, scheduled and dispatched tooling explicitly binds
`VIZE_LSP_BIN` to the current workspace's `target/ci/vize` and requires its
source-build receipt. Validate the exact path, current source revision,
workspace version and binary hash before probing or spawning that binary.
Missing, stale, corrupt or unlaunchable required bindings fail without trying
a cached debug executable. Standalone launch discovery retains its existing
fallback when required source binding is absent.

Reuse the receipt emitted after `cargo build --profile ci -p vize`; add no
pipeline stage or second build. Executable temporary-Git tests include both a
runnable old debug binary and a fresh CI binary, then mutate the bound binary,
receipt and source revision to verify rejection before any fallback.

Print that existing public receipt immediately after its successful creation
so completed Actions logs retain the exact source revision, binary path, hash
and workspace version. Retain no inferred hash from earlier unlogged runs;
this adds neither a second receipt nor another workflow or build stage.

This closes an evidence identity gap, not a native product acceptance gap.
Full runtime fixture Actions must still execute with the bound current binary;
PR T0 deferrals, unhydrated corpus limitations and required T1 work remain.

A controlled VP task-route check imports the actual root configuration and
keeps its cached-script setting and unchanged `test:scripts` task definition.
All five Node test workers receive both binding variables and execute the fresh
receipted fixture binary. Removing its receipt makes every worker fail despite
a runnable older debug binary. The source-built fixture dispatches still need
their own exact-head runtime proof; this route check supplies no product credit.
