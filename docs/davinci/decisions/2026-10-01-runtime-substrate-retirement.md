# Retire the compatibility runtime package (2026-10-01)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833), after
the runtime, consumers, owned pages and law harnesses have their real owners.

The last legacy consumers import shared contracts directly from L0. This
removes their `vize_davinci` declarations, normal or dev, without replacing
a product route. Croquis CF adds its explicit L0 dependency; Carton remains
its legacy storage facade in `crates/`. Diagnostics use `vize_l0::diag` and
witnesses its verifier; pass/fact, key, identity and Dump APIs retain the same
implementation. No legacy fixture or captured historical output is rewritten.

Two microbench files move alone into L0 as `pass_runtime` and `fact_runtime`.
Their functions and suite selectors follow those names. The four existing
measurement IDs and all budget values remain pinned so instruction and exact
allocation comparisons retain their baseline. File provenance follows the
new bench paths; this introduces no pipeline stage or serialization.
The four source identities are rebound from the actual [Actions measurement](https://github.com/ubugeeei-prod/vize/actions/runs/36778058361),
source `001f3eee417b1522392233bf79b00f8fe32cf3d7`. All 100 probes produced
three identical measurements and met the existing numeric ceilings. Pass
counts stay 5/5 and fact counts are 10,841/10,692 below 11,290/11,141;
only the four moved source paths and their hashes change in the registry.

Only the five reviewed facade files are deleted: Cargo manifest, README and
three re-export modules. The workspace/lock and publish order drop the package;
Source inventories and the public Rust support table drop its obsolete surface. A live metadata law prevents
the retired package or any declared dependency from rejoining the workspace.
Old identities in frozen captures, replay maps and synthetic historical graph
fixtures remain evidence, rather than active imports or package declarations.

The plugin host build ID hashes the actual L0 and derive manifests and source
files, replacing the retired source root. Dirty foundation or macro edits must
still invalidate the native host identity; missing sources remain build errors.
The retirement replay includes this source-closure update.

TS-24 still builds six actual library packages: std L0 and five native level/
conversion source libraries. All five retain their `no_std`/alloc attributes.
The accepted std foundation was already transitive; the lane now names it
directly instead of building its empty compatibility facade. The workflow,
normative suite row, source/metadata law and portability ledger change together.
This is WASI-with-std evidence, not completion of #6834 or embedded support.

Replay `python3 tools/support/levels/retire-runtime-substrate.py moves`, commit
the two file moves alone, then run `integrate`, format Rust, normalize locked
metadata and regenerate inventories. Move collisions, changed facade blobs,
unexpected files and partial retirement are rejected before writes. Consumer
rewrites are buffered and retirement removes only the reviewed files.

Local validation passed CLI/Croquis-CF checks for all targets, the actual DOM
profiling harness, retained stage-edge/release-firewall, layout, storage and
portability workflow laws. Two obsolete registry expectations were corrected
to their retained owners and passed focused checks. Exact-head Actions, strict
instruction and allocation gates, full corpus and actual protected-queue merge
remain required; this local change does not close #6833 yet. Higher-level
skeletons, native providers, product routes and performance acceptance remain
unfinished in the completion ledger.
