# Rust cache backends

Reviewed candidate for [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).
Actual trusted Actions seeding, later source/queue cache hits and fresh runtime
reports remain pending. The paired issue comment is drafted below.

## Decision

PRs, merge groups and every nondefault-branch checkout use the existing pinned
Actions restore-only action; they cannot save into any branch namespace. Only push, schedule or manual dispatch on the actual
repository default branch may use the existing stable Blacksmith namespaces.
The checked-out commit must equal the event SHA; a manual dispatch that checks
out another commit uses Actions cache. GitHub-hosted runners use Actions cache.
Only that trusted context can save Actions entries, including on GitHub-hosted
runners. Pull-request-target, workflow-run and repository-dispatch contexts are
restore-only even if their event ref names the default branch. Unknown events
and malformed repository/ref/SHA metadata fail before any mount.

The compatibility prefix input remains accepted, but no per-PR-number or
per-candidate-SHA Blacksmith key is created. Stable provider registry, Git and
target-role keys keep their existing names. Actions keys include repository,
actual runner OS/architecture, suffix, Cargo.lock/toolchain bytes and target role. Target entries also
include the checkout SHA, with a compatible role/input prefix for restoration.
An optional second target stays distinct and includes its own lock/toolchain
when it belongs to a separate benchmark checkout.

Trusted stable sticky clones seed Actions cache. The provider steps precede
the cache steps. A successful actual mount uses Actions `lookup-only`; it does
not extract another cache into the mounted path. The pinned cache action saves
a missing seed after successful validation, before the earlier provider post
steps unmount their clones. Each path is probed separately: a failed provider
mount falls back to ordinary Actions restoration for that path.

Cache hits never replace validation. Existing Cargo builds, archive/source
receipts, doctests, selected/full tooling, differential suites and four Rust
workers retain their current commands. No upstream SDK, provider API, external
deletion or billing configuration changes are part of this preparation.

## Evidence and limits

- #6952 source f41c2105 / Check 36321231094 selected 525/631 files, deferred
  106 known T1 files, and ran 4855 cases: 4843 passed, 12 known skips, no failures
  or cancellations. Node execution was 316.221 seconds; the tooling job took
  471 seconds. Parent 274a87b4's Node execution was 318.159 seconds.
- All three source-job mounts failed at the provider's 1000/1000 limit. The
  dynamic namespace adds registry, Git and four target-role keys per PR or
  merge candidate when every lane runs; there is no lifecycle cleanup.
- Existing stable registry/Git/tooling-target keys mounted and committed in
  full dispatch 36317762656 (12:04–12:16 UTC), after new CI4 candidate keys
  already hit the limit. Existing keys can be reused without external cleanup.
- Pinned stickydisk d58829ad clones a snapshot and writes it back in successful
  post. It has no restore-only input. This change therefore keeps untrusted
  jobs away from those writable provider keys rather than sharing them.
- Local tests execute the real policy/composite shell step and distinct fake
  backend children: many PR/queue/dispatch contexts, a real fixture artifact
  built after both cache hits and misses, actual mount-probe fallback, clone
  preservation, seed/unmount order and malformed-event rejection. Existing
  build/test workflow commands stay unchanged. Fakes do not execute the real
  suites or prove live cache service
  permissions, transfer time, retained entries or actual Actions post behavior.

TODO: verify a trusted default-branch seed followed by a PR restore
and all fresh runtime/source reports. No CI p50/p90 or wall-time gain is claimed.
Actions cache's restoration scope and eviction apply independently of the
provider disk limit; initial cache misses still build and test normally.

The current full Check has trusted target writers for `test-scripts`,
`test-js-packages`, `playground-test` and `clippy-test`. PR-only `nextest-ci`
currently has no trusted default-branch target writer. Its registry/Git entries
can use the shared trusted seed, but its exact target-role seed remains a TODO;
this slice does not alias target roles or add a build job. Full Check's legacy
Rust recipe uses the dev profile while PR/queue archives use the ci profile, so
a mounted target alone does not establish a warm archive build.

The primary target key uses repository lock/toolchain inputs. The standalone
`tests/fuzz/target` workspace has its own manifest and generates an untracked
lockfile, so those standalone inputs are not fully represented by this key.
The unchanged Cargo commands still resolve and validate the current inputs;
a restored target does not substitute for a fresh build.

## Issue #6830 comment draft

The reviewed cache candidate bounds future provider names by routing PRs,
merge groups and nondefault branches to restore-only Actions cache. Only an exact trusted
default-branch checkout writes existing stable Blacksmith keys, and those
clones seed Actions without a second restore into their mounted paths. Failed
mounts fall back per path. All validation remains required; external cleanup
is unnecessary for the new policy. The source record is
`docs/davinci/decisions/2026-09-27-rust-cache-backends.md`. Actual
seed/restore/runtime proof remain unfinished; this does not close the CI issue
or establish the three/six-minute target.
