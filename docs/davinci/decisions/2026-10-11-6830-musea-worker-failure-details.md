# Preserve Musea test-worker failure details

This decision belongs to [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Musea's existing `tsx --test` command now sends Node's standard spec reporter
to stdout and its standard TAP reporter to stderr. The existing Vite build,
runner, file patterns, native guards, test cases, and deadlines stay intact.
There is no additional workflow, test stage, or custom reporter.

The original [JS package job](https://github.com/ubugeeei-prod/vize/actions/runs/38127324554/job/114434407903)
checked out `3c3621d4ca72351b57e52516368722d7a91524f3` and failed
`src/variant-bindings-http.test.ts` as an anonymous whole-file failure. Its
spec output gave `test failed` without an exception, exit code, or signal.
The package reported 313 tests: 283 passed, one failed, and 29 skipped.
The native HTTP preview case was skipped separately. None of these outcomes
proves a particular process exit, signal, native defect, or timeout.

Node's test runner attaches an `exitCode` and `signal` to a worker's
`testCodeFailure`; its spec reporter omits those fields and TAP retains them.
The regression control reads the package's actual runner options and runs
real TypeScript workers that exit with code 7 or terminate with `SIGTERM`.
Both invocations must still exit unsuccessfully, and their TAP stderr must
retain the appropriate original field. These deliberately failing workers
test reporting only; they do not explain the original HTTP failure.

The original whole log is 1,259,112 bytes, with SHA-256
`3af127370c567b2c44425310a5bd6b1043f56540aeffc4818f8d918240df1f6e`.
Its three virtual-module `ERR_LOAD_URL` messages also occur when the original
whole HTTP test passes locally, so they do not establish the CI failure's
cause. The unchanged local test passed under Node 24.14.0 and tsx 4.23.12;
that macOS result is separate from the failed Linux job.

The necessary next step is the ordinary source Check on this change, using
the original Linux package suite and its existing budgets. If the anonymous
failure recurs, retain its TAP details and investigate the actual failing
worker. If it does not recur, preserve the original failure as unresolved;
additional reporting alone does not prove a product repair.
