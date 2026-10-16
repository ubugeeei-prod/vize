# Musea HTTP lifecycle diagnosis

The synthetic middleware test uses a private empty Vite root and cache. Its HTTP
routes do not need the real gallery, dependency graph, or another test's cache.

Run a bounded comparison on Linux with the repository Node and frozen dependencies:

```sh
vp exec node npm/builder/vite-musea/scripts/http-lifecycle-probe.mjs BASELINE_COMMIT OUTPUT_DIRECTORY
```

`BASELINE_COMMIT` must be a full 40-character commit containing the original test.
The command executes the original and current HTTP contracts twice each, retaining
all route assertions. It adds synchronous phase observations to temporary copies,
preloads native-library identity recording, and enables Vite dependency scan logs.
The output includes authored/instrumented source hashes, complete stdout/stderr,
PID/thread identities, loaded native hashes, and phases through Vite shutdown.
A nonzero result, signal or timeout fails the command after every packet is saved.
The temporary test copies are removed; this does not rebuild native code.

The controlled [Linux run 38902115591](https://github.com/ubugeeei-prod/vize/actions/runs/38902115591)
on Node 24.14.0 and Vite core 0.1.24 observed the original test scanning
`gallery/index.html` from the package root. Both isolated runs used distinct empty
roots/caches and did not scan that gallery. All four runs passed their HTTP
assertions and reached Vite shutdown completion. Only the Vite+ native binding was
loaded by the target workers; Vize native was not loaded. Because the baseline
also passed, this establishes fixture isolation, not elimination or the native
cause of the intermittent SIGSEGV tracked in the failing builders jobs. Do not
accept a crash after partial output or apply the separate post-pass exception in
issue #9067 to this test.

The unchanged test worker had previously received SIGSEGV in these independent
builders jobs:

- [PR #9214, run 38891927465](https://github.com/ubugeeei-prod/vize/actions/runs/38891927465/job/116943368794)
- [PR #9239, run 38898004961](https://github.com/ubugeeei-prod/vize/actions/runs/38898004961/job/116963193228)
- [PR #9248, run 38900756342](https://github.com/ubugeeei-prod/vize/actions/runs/38900756342/job/116973297995)

The retained artifacts from the first two failures contain identical test,
middleware, lockfile and Vite+ native hashes, despite different Vize native builds.

The four controlled successes were baseline workers 5648/5746 and isolated
workers 5835/5920, all Node 24.14.0. Each loaded only
`vite-plus.linux-x64-gnu.node` with SHA-256
`40191de50fdb094bda9ee76e3c4ef35ff29c505e8b1fc7de563b51facdc6d73e`.
These bounded observations do not identify the crashing native instruction or
prove that concurrency was causal. The earlier whole-builders process-map
artifacts lack an authenticated mapping from the failing test name to its PID,
and core capture was unavailable, so they cannot supply that missing proof.
