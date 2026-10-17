# Stop superseded full workers while retaining failed evidence

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The superseded [full Check run 38890273485](https://github.com/ubugeeei-prod/vize/actions/runs/38890273485)
was canceled before its replacement was created at 10:55:51 UTC on October 16, 2026. All sixteen old `Tooling (...)` jobs nevertheless started at
10:56:14–10:56:15 UTC. The worker job used `always()`, which intentionally
bypassed producer failures but also let canceled work start. Its independent
native captures repeated the same status condition inside composite actions.

Use `!cancelled()` for the full tooling worker and its independent execution
boundaries. This status function still permits a failed or skipped producer,
so its independent observations survive a preceding failure. It becomes false
when the run is canceled. Apply the same boundary to the full JS package test
call and the nested plugin setup, attribute captures, selected static-class
captures. Keep each existing shard predicate.

Evidence uploads retain `always()`. The full tooling and full JS aggregate jobs
also retain their existing `always()` conditions and reject every failed,
canceled, skipped, or missing required producer. A canceled source build cannot
turn into successful release qualification. Ordinary steps retain their existing
success gating; test commands, exhaustive inventory, matrix partition, producer
identity, timeouts, and full qualification event selection stay unchanged.

The regression evaluates the actual parsed job and step expressions across
scheduled/manual and unrelated events, successful/failed/skipped predecessors,
both shard roles, and both cancellation states. It also asserts unchanged
upload retention and strict aggregate rejection. Existing full-tooling tests
still prove that the shard union equals the complete test inventory. These
source contracts prevent restoration of the known condition bug; actual Actions
cancellation behavior must also be observed after this change is deployed.

The current-main successor preserves its three existing full-worker shard hooks.
The dormant historical-SFC action and its absent caller are separate from this
change; the obsolete stack's additional full-worker capture hook is not imported.
