# Reuse pnpm's verified lockfile record

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

At `986d6cdb14d866bf23786099acbc4f57dff5c824`, the
[Rust CLI cohort](https://github.com/ubugeeei-prod/vize/actions/runs/38893209547/job/116948362831)
spent 77.1 seconds in setup-vp despite hits for its 92 MB bootstrap and 619 MB
package-store caches. Installation reused 1,915 packages and downloaded zero.
Verifying the lockfile's 2,333 entries took 70.6 seconds; one metadata request
for `@ox-content/binding-win32-x64-msvc` took 70,062 milliseconds. The policy
verifier checks the whole lockfile, including optional packages for other
platforms. Narrowing installed packages does not remove that verification.

The pinned setup-vp action caches the directory returned by `vp pm cache dir`,
which Vite+ 1.3.0 maps to `pnpm store path`. pnpm's metadata cache is separate.
The official [pnpm CI guidance](https://pnpm.io/continuous-integration)
recommends discovering it with `pnpm cache path` and caching its verification
log, provided only trusted jobs can populate caches used by trusted consumers.

## Native verification remains authoritative

The action uses `vp env exec pnpm cache path`, so it works before workspace
dependencies exist and honors configured paths without guessing the platform's
default. Only `lockfile-verified.jsonl` is restored. pnpm 12.1.0 generates and
reads that file itself; the action never synthesizes a successful verdict.

The reviewed source is pnpm tag `v12.1.0`, commit
[`6d90c71efdffbc909b499490b64c66badc720327`](https://github.com/pnpm/pnpm/tree/6d90c71efdffbc909b499490b64c66badc720327/pnpm/crates/lockfile-verification).
Its content hash permits reuse at a different checkout path. Every active
verifier must accept the recorded policy. Tightened age limits, changed trust
policy or exclusions, and missing integrity/binding markers invalidate the
record. Invalid records fall back to verification, and failed verification
does not record success. Dependency-name validation precedes lookup. This uses
pnpm's existing content-and-policy cache lifetime, which has no separate TTL.
See the [pinned policy comparison](https://github.com/pnpm/pnpm/blob/6d90c71efdffbc909b499490b64c66badc720327/pnpm/crates/resolving-npm-resolver/src/create_npm_resolution_verifier.rs#L332).

The external cache key additionally binds the exact lockfile, root manifest,
workspace configuration, actual pinned pnpm version, and an allowlist of active
policy and resolved registry settings. The supported `config list --json`
command merges configuration and environment overrides; its resolved registry
map includes default, scope and named routes. Credentials are excluded from
the identity and output. See the [pinned configuration reader](https://github.com/pnpm/pnpm/blob/6d90c71efdffbc909b499490b64c66badc720327/pnpm/crates/cli/src/cli_args/config.rs#L639).

Unknown settings, dynamic pnpmfiles/config dependencies, alternate lockfile
locations, unsafe paths and version mismatches skip this additional cache.
The existing install still runs. No age, trust, integrity or installation flag
changes; in particular, this does not enable `trustLockfile` or offline mode.

## Trust boundary and integration

The existing `rust-differential-target-seed` job is the only configured writer.
It performs its existing full installation once. Saving requires an explicitly
requested trusted default-branch event, exact checked-out event SHA, and a
successful job through actions/cache's post step. PR Rust cohorts, PR JS groups
and conformance checks, and the shared full JS setup use restore-only steps.
There are no broad restore prefixes.

Verification records are trusted cache contents, not signed attestations.
[GitHub's cache ref isolation](https://docs.github.com/en/actions/reference/workflows-and-actions/dependency-caching#restrictions-for-accessing-a-cache)
prevents PR merge-ref entries from being restored by trusted default-branch
jobs. The new action does not grant PRs or merge groups permission to save this
cache, and does not cache authentication configuration or registry credentials.

## Validation and remaining measurement

All 31 focused tests passed, including the affected Rust/JS workflow contracts.
The real pinned pnpm test uses a loopback registry and frozen lockfile-only
installs: cold verification creates a record; copying only that file into a
fresh checkout produces `cached` with zero HTTP requests. Tightened minimum
age and a provenance trust downgrade fail; removing tarball integrity fails;
corrupting the record causes normal verification again. Failed checks leave
the successful record unchanged. The native test took approximately 0.39 s.

Run the new controls with
`node --test tests/tooling/pnpm-verification-cache*.test.ts`.
Narrow formatting/lint and strict native TypeScript checks passed, and
actionlint qualified all 93 workflows.

This is a new cache namespace. It remains cold until the trusted main seed
completes successfully. The local controls establish behavior, not hosted
latency: a subsequent exact-key Actions restore must show pnpm's
`passes supply-chain policies (verified ... ago)` result and a measured install
duration. No 70-second reduction or five-minute whole-PR result is claimed yet.

At source `986df9a534e18941da8d23cb90ed2bf8d733f355`, six completed setup
observations from [Check attempt 1](https://github.com/ubugeeei-prod/vize/actions/runs/38899025329/attempts/1)
and [Musea](https://github.com/ubugeeei-prod/vize/actions/runs/38899025194) all
resolved the same valid exact key, requested `save=false`, skipped the writer
and missed the cold namespace. Discovery took 91–156 ms and restore lookup
198–219 ms. Normal policy verification still checked all 2,333 entries; installs
took 3.2–3.7 seconds with zero downloads. This confirms actual hosted configuration
compatibility and read-only operation. The earlier 70-second metadata request
did not recur, so its absence is not attributed to this still-cold cache.

## Native test runtime

The exhaustive tooling task exposed a different `vp` on PATH than the isolated
test: workspace Vite+ 0.1.24 has no global `env` command. Prepending the real
workspace binary directory reproduces the hosted `Command 'env' not found`.
The native test now invokes the pnpm already selected by `vp run`/`vp exec` and
requires its reported version to exactly match the root `packageManager` pin.
The same PATH and environment serve every policy probe. The cache action's
pre-install global-Vite+ invocation remains unchanged.

`COREPACK_ENABLE_NETWORK=0` prevents a different PATH from downloading a separate
package manager. An empty Corepack cache fails explicitly; a wrong pnpm version
fails before installation. Neither case skips the test. The original synthetic
manifests and frozen lockfile are retained: adding a manifest package-manager
field would require a new pnpm package-manager dependency section in that lockfile.
The actual managed-runtime invocation passes all existing cold/warm, age, trust,
integrity and corrupt-record assertions with the original install flags.
