# L0 leaf queue order (2026-09-28)

## Decision

The maintainer placed [#7043](https://github.com/ubugeeei-prod/vize/pull/7043) in the merge queue at 08:29:35 UTC while [#6832](https://github.com/ubugeeei-prod/vize/issues/6832) was open. Deliver this independently validated L0 package, id, and side-table slice before #6832 closes. This is an exception for #7043 alone. The roadmap order remains the default for later slices, and #6833 and #6834 stay open until their remaining work is done.

## Merge conditions

- Refresh #7043 against current `main` and any preceding queue entries. A stale `UNMERGEABLE` entry must be removed before pushing a repaired head, then re-enqueued.
- Run the ordinary PR checks on the exact repaired head. In the protected merge queue, keep the instruction ceilings and full suites; do not raise budgets.
- Use squash merge and verify both terminal merge-queue checks and the resulting commit on fresh `main` before marking the PR delivered.

The package alias resolves to physical `vize_l0`, which currently re-exports Carton's public storage API. The independent storage carve-out remains [#6834](https://github.com/ubugeeei-prod/vize/issues/6834). The rest of the substrate move and deletion remain [#6833](https://github.com/ubugeeei-prod/vize/issues/6833).
