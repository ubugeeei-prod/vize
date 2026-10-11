# Prepare the pinned Element Plus fixture for workspace source coverage

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

`coverage:source` invokes the original `cargo llvm-cov --workspace` task. That
executes `vize_canon`'s native syntax integration suite, including the whole
Element Plus popover/tooltip projection law. Its original pin assertion requires
`20298a21ae54a5417955373b80130ae7c548a6bb` and compares the entire original
fixture sources before checking both complete generated projections.

The full Check at source `750a9d47d12c3f813bf2e9055a85af6058197ddf`, run
[38182228212](https://github.com/ubugeeei-prod/vize/actions/runs/38182228212),
failed in source-coverage job `114597016866`. The original assertion found the
enclosing Vize HEAD instead of the frozen Element Plus pin, because checkout
left that gitlink uninitialized. Preserve that real failure. The merged
[full native prerequisite](./2026-10-12-6830-full-native-element-plus-hydration.md)
prepares another runner; it does not hydrate this coverage job.

Extend only source-coverage's existing Pkl installation step with the same
`git submodule update --init --force --depth 1` command and the exact fixture
path. Pkl installation must succeed first, and fixture preparation must fail
normally before coverage if the original upstream input is unavailable.
The submodule update reads the existing repository and fixed gitlink. It makes
no upstream changes and does not select the upstream branch's current HEAD.

Branch coverage has a distinct authored task: `cargo +nightly llvm-cov` selects
only `vize_carton`, `vize_armature` and `vize_atelier_core`. It does not execute
the `vize_canon` pin law, so adding this fixture there would be unused setup.
Its entire job and task remain unchanged. Parsed workflow/task laws retain this
consumer distinction; a future workspace expansion must prepare its required
fixtures in the same runner.

The regression executes the actual workflow's preparation command against a
real local Git submodule. It reproduces the empty directory's enclosing HEAD,
restores the older fixed gitlink while the upstream HEAD has advanced, repairs
a wrong and dirty checkout, refuses an unavailable upstream and verifies the
upstream HEAD remains unchanged. Counterexamples also reject missing,
conditional, nonblocking and semicolon-separated preparation. The existing
actual repository gitlink and original native pin assertions remain checked.

All coverage commands, selected tests, compiler/instrumentation choices,
thresholds, timeouts, Nuxt iteration environment, summary validation and uploads
remain unchanged. The complete adjacent workflow laws pass locally without
building Rust. Exact-head Actions and the original full source-coverage job
must independently run this preparation and native suite; protected queue and
actual merge remain required. Preparation and a green PR are not full coverage
or release evidence, and this record makes no performance claim.
