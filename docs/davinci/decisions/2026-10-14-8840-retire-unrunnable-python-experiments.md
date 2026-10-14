# Unrunnable Python experiments are deleted, not ported

This [#8840](https://github.com/ubugeeei-prod/vize/issues/8840) record follows
the [Python retirement ledger](./2026-10-13-8840-retired-tooling-languages.md).
Two manual measurement experiments can no longer run, so their Python is
deleted with them instead of being ported to TypeScript.

- **Allocation ownership (#7764).** The `check.yml` job ran only on manual
  dispatch from `ci/vapor-allocation-ownership-20261004`. That branch no
  longer exists, #7764 is closed, and its
  [record](./2026-10-04-vapor-allocation-ownership.md) authorizes no further
  trial. The job, its dispatch input, the `diagnose-allocation-ownership`
  action and the six `allocation_ownership/*.py` files are deleted. The
  opt-in `allocation-window-diagnostics` feature and every allocation gate
  stay.
- **Rust archive window.** The
  [trial](./2026-10-12-rust-archive-window-trial.md) reads the frozen
  artifact `11706049605`, which has expired (the API returns 404). The
  `rust-archive-window` walltime choice, its reusable workflow, the
  `measure-rust-archive-window` action, its workflow test and the four
  `rust_archive_*.py` files are deleted. The artifact-payload trial keeps its
  own producer and restore actions.

No required check runs these experiments. Their scripts stay readable in git
history at the parent of this change, and the records above keep describing
the original runs.

Python files: 42 → 32.
