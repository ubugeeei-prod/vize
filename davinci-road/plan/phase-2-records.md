# Phase 2 — Task records

> [!NOTE]
> What each landed phase-2 task actually measured, decided and left open, one
> file per task. The **contracts** are in [phase-2-tasks.md](./phase-2-tasks.md)
> and the phase-level record — the re-cut, the phase-1 carry-ins, the TODO
> index and the exit gate — is in [phase-2.md](./phase-2.md).
>
> Records are separate files for the reason the contracts split from the phase
> file: the repository's 350-line source-length budget
> (`tools/moon/cmd/source_file_lengths --max-lines 350`), which plan files are
> not exempt from. A record grows with its task, and 22 of them cannot share a
> page.

| task                                  | landed     | what it decided                                                                                                        |
| ------------------------------------- | ---------- | ---------------------------------------------------------------------------------------------------------------------- |
| [P2-1](./phase-2-records/p2-1.md)     | 2026-08-19 | `NonZeroU32` node ids; sparse-only side table with the densification trigger written down; owned `'static` diagnostics |
| [P2-2](./phase-2-records/p2-2.md)     | 2026-08-19 | const-data pipelines; both pass-manager laws enforced as compile errors, both proven by compiling a violation          |
| [P2-3](./phase-2-records/p2-3.md)     | 2026-08-19 | the fused-group reporting law; static dispatch so the un-observed path has no check at all                             |
| [P2-12a](./phase-2-records/p2-12a.md) | 2026-08-19 | the pre-S2 traversal baseline, the phase-2 target, and the plan finding that corpus `--check` is not evaluable         |
