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

After CI PRs #6918, #6958, #6919 and #6925 actually merged at 11:37:59 UTC,
replay the reviewed binding and receipt logger onto main
`42014675678684165b8cd4bb992bb9db747e74ee`. Preserve the actual full-tier guards,
instruction gate and shared inventory composite. The earlier #6938 Check
`36308131243` at `b4671e34a` is historical; require new-head Actions and fresh
full fixture dispatches with the printed receipt before claiming acceptance.

Compose the existing binding with the formatter corpus uploader by placing
the required variables on the full tooling test step and preserving its plain
VP command. Earlier build and preparation steps retain their environment.
Keep the uploader after that command when the queued formatter changes merge;
do not import its unmerged files into the binding repair. Neutral YAML map and
comment compaction keeps the grandfathered Check workflow from growing even
after composing both sets of changes.
Verify parsed semantics, the actual configured VP worker route and clean merges
against both actual main and the formatter queue prefix before republication.

The tested checkout of each stacked fixture includes main's formatter uploader.
Its source-length comparison must therefore start from a parent that already
contains the same 690-line workflow, rather than a 688-line source parent whose
synthetic merge grows by two lines. Preserve the uploader and ratchet unchanged.
Compose the binding on the published remaining linter chain ending at
`0c7043a45ee656d17c94aca7aa8301124f0f53bd`, which includes actual main
`30c5af859a4820461450ba028508c886c92e986a` and the tracked-Git-list streaming
repair. Inherit the entire parent tree; do not copy that unmerged helper patch.
Replay the approved LSP fixtures and Vue dialect repair only after this parent
is reviewed. Require fresh functional checks and the full native queue's named
source-bound runtime proof before claiming acceptance.
