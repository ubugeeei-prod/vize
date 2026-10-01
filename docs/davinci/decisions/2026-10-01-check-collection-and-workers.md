# Check collection and checker workers

Issues: [#7324](https://github.com/ubugeeei-prod/vize/issues/7324), [#7325](https://github.com/ubugeeei-prod/vize/issues/7325).

## Collection

Within one collection, a fully scanned alias closure with no Vue dependency proves every visited member does not need a virtual registration. Reuse that negative result for other alias entries and for suffixes of later walks. Never propagate a positive answer to every member of a mixed graph: unrelated branches can remain plain TypeScript.

Keep package-aware walks separate. Their transitive sources belong to the importing package route's invalidation inputs, so a plain-alias memo must not suppress those sources. Start a fresh memo for the next collection, preserving visibility of edited files. Existing package-route reachability caching, ordering, watch inputs, and bounded scan budgets remain intact.

Register the reported synthetic alias chain at 100, 300, and 600 SFCs with twice as many TypeScript modules and eight imports per SFC. Check actual source-read work (200, 600, and 1,200), complete authored source membership, no unnecessary registrations, a mixed Vue-positive graph, a changed leaf in the next collection, and package invalidation isolation. The owning Vue fixture lives under `crates/vize/tests/fixtures/import-registration` and is consumed by source-walk and CLI tests.

## Workers

Expose `vize check --checkers N` and per-executor Rust setters. Explicit positive counts override `VIZE_CHECKERS`; absence retains that environment option and the deterministic one-worker default. Apply the selected width consistently to single CLI runs, each shard, and CLI fallback. Automatic server sizing accounts for workers per process.

Keep the default from #3905: the existing npmx.dev/tsc comparison recorded missing and different diagnostics with wider Corsa pools. Increasing the default would trade correctness for throughput without resolving that discrepancy. Zero and malformed CLI counts fail before starting Corsa. An unsupported runtime still fails instead of retrying without the option.

Register source-built CLI subprocess coverage for default, environment, explicit override, disjoint shards, and rejected counts. Update the complete CLI help snapshot. Validate with GitHub Actions; do not relax diagnostic or scan budgets.
