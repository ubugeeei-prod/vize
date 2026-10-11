# Preserve real-project native LSP wire responses

Owner: [#3952](https://github.com/ubugeeei-prod/vize/issues/3952).

The original enforced Matrix run
[38156816230](https://github.com/ubugeeei-prod/vize/actions/runs/38156816230)
at `8dc523810ed4752f8132de72dd3b645eb75ad39d` retains all 22 report packets,
145 projects, 41 authored projects and 246 anchors. Eleven LSP contracts fail.
Those reports retain hover contents in assertion messages and abbreviated
reference diffs, but cannot establish complete responses or physical ranges.
The existing launch was not source-required, so its passive raw collector was
inactive. No missing packet is inferred or reconstructed from a failure diff.

The same LSP step now records the already-built CLI with the existing receipt
writer, authenticates its literal source, executable digest, version and
unchanged `cargo build --profile ci -p vize` recipe, and enables the existing
passive session collector. It observes the original client/server/stderr
streams and terminal status from those actual processes. It sends no extra
protocol requests and starts no replacement server. Existing collector byte
limits and explicit truncation/failure markers remain unchanged. Missing,
truncated or incomplete responses cannot qualify a native oracle successor.

A separate artifact preserves raw streams, decoded messages, process identity,
terminal records and the CLI build receipt for each exact source and shard.
The original shard summary artifact, selected fixture pins, all 22 shards,
contracts, assertions, timeouts, budgets and surface gates are preserved.
Captured lifecycle timings include passive recording and are labeled separately
from paired performance acceptance. This change grants no failure waiver,
source fix, successful lifecycle or release credit.

TODO: use complete native hover/reference/range packets and the unchanged
pinned sources to diagnose each original failure. Qualify any finite semantic
successor through a separately reviewed source-backed contract; then rerun
all original lifecycles and complete release acceptance.
