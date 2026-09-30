# Croquis owns its dump page (2026-10-01)

Decision for [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).

The historical Croquis page belongs to `vize_croquis::dump`, alongside its
producing `Croquis::to_vir()` renderer. Its parser, printer and structural
model move together; the old substrate has no Croquis page re-export. Shared
Dump, fact and pass contracts come directly from L0, so Croquis also drops its
normal dependency on the compatibility substrate. Native level crates gain no
legacy dependency and no product compile path changes.

The move-only commit relocates 38 files: four implementation files, four law
harnesses, fourteen unchanged Vue/folio fixture pairs and two unchanged
snapshot files. Full/Display printing, grammar errors and normalization retain
exact bytes. The SFC fixture harness uses the same parser and allocator via
their direct owners. Its version-less, path-only SFC dev dependency is available
on every test target; host-only benchmarks remain target-gated. No acceptance
law is disabled to accommodate the move.

The isolated fuzz harness imports the Croquis page from its new owner and L0's
Dump trait directly. Corpus seeding includes the moved folios in addition to
the remaining old-substrate and L2 fixtures. Source witnesses, storage scope,
current test commands and local links follow their owners; captured snapshot
metadata stays frozen.

Replay `python3 tools/support/levels/move-croquis-dump.py moves`, commit source
moves alone, then run the script with `integrate`, format Rust, normalize Cargo
metadata and regenerate source inventories. Move preflight rejects collisions
before the first index mutation. `check` confirms no source was left behind.

Local verification passed all sixteen fixture, parser/printer, error and
pipeline snapshot laws, including live SFC analysis of all fourteen inputs.
The three source inventories match their current scans. Actions and protected
queue validation remain required before merge. #6833 remains open for
feed/repro pages, legacy test plans, other consumers and final substrate
deletion; this slice does not complete native L1-L4 or product migration.
