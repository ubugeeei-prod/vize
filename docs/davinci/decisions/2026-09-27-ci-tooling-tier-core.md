# Tooling input wiring and queue inventories

Tracked in [#6863](https://github.com/ubugeeei-prod/vize/issues/6863),
[#6864](https://github.com/ubugeeei-prod/vize/issues/6864) and
[#6865](https://github.com/ubugeeei-prod/vize/issues/6865).

Use the tooling selector's explicit GitHub boolean output to select the lane.
The PR job fetches the same proven comparison base and regenerates its exact
test plan before executing the selected serial task. Keep source-built CLI
receipts, native binding, plugin isolation and MoonBit prerequisites. Merge
groups retain the complete tooling task and all explicitly deferred runtime
scenarios. Unknown runtime requirements remain fatal.

Keep PR browser tests. Run VRT, its failure artifacts and failure rejection in
the merge group. JS package selection and workload retain their existing work.
Source inventories and complete JS checking run in merge groups, schedules
and dispatches; PRs retain fast repository checks. Security audit selection and
generated ledgers belong to independent PRs and are excluded from this child.

This child builds on the separately validated Rust archive/shard branch.
Tooling selector code matches the independently reviewed selector PR, with
its original baseline inventory counts pinned to the original commit. Actual
latency and native completion require Actions and product evidence.

Focused verification: 24 tooling-selection, receipt, queue and workflow tests
pass with no skips or cancellations. Root `vp check` passes on all nine changed
JS/TS/MJS files with zero warnings or errors. Both changed workflows pass
actionlint; the 350-line growth ratchet passes against Rust parent `016ba456f`.
Fresh Actions runs and full merge-group validation remain required.

After #6901 merged at `dd047952c`, this child is transported onto the full
workspace shard layer `aed02f464` above repaired core `138c62019`. Preserve
main's existing inventory composite and generated-ledger upload while applying
the merge-group/schedule/dispatch guard; do not restore standalone summary
commands. The full profile remains unfiltered with required TSGO and all
differential recipes. Its receipt rejects even an empty runtime opt-out; the
full shard environment omits that variable completely. Transport verification
passes 23 workflow, selector,
receipt, runtime-tier and producer-retry tests, strict checks on the three
changed test files, actionlint and the source-length ratchet. Latest-head
Actions and the atomic three-PR merge queue remain required.

Main's #6942 action-directory move is transported in its original separate
move and caller commits. The action bytes and artifact contract are identical.
Merge actual main ancestry to retain that level-only path with the full-tier
guard; merely replaying the move still conflicts at the adjacent guard. Keep
the instruction-count gate mandatory when its preceding queue reaches main.
