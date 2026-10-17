# Refresh the PR native build cache after source merges

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The JS builders and UI jobs in PR #9200 used the same read-only
`test-js-packages` sticky disk across three heads. Each build reported 435
Cargo artifacts: the same 425 fresh artifacts and 10 rebuilt workspace
artifacts, with unchanged package, feature and profile identities. Cargo's
native build took 31–42 seconds in the earlier runs and 141 seconds in both
jobs of [run 38889929446](https://github.com/ubugeeei-prod/vize/actions/runs/38889929446).
The timing variation remains unexplained; the receipts establish that these
were equivalent builds, rather than a profile or cache-key change.

The existing main seed refreshed `nextest-ci` and `clippy-test`, but neither
is the JS jobs' dev-profile disk. Its regular trusted writer was the scheduled
or manually dispatched full Check. PR and merge-queue clones cannot publish
their changed dependency builds, so another candidate keeps rebuilding those
dependencies until a trusted writer catches up.

Add one independent native build job to `rust-pr-target-seed.yml`, retaining
its main-push and daily scheduled triggers. It restores the Vite+ bootstrap
cache, reuses `setup-js-package-tests`, then runs
`vp run --workspace-root build:native:test`. That shared action owns the stable
Rust toolchain, Wild linker, frozen JS installation and `test-js-packages`
cache namespace. The Blacksmith runner and `CARGO_BUILD_JOBS=12` match the PR
package jobs. The task retains the dev profile, `napi,legacy` features and
`--no-js` wrapper preservation. Native sources, build task helpers and setup
changes also trigger the seed.

The existing cache policy only commits a trusted default-branch checkout.
PR and merge-queue mounts stay read-only; the scheduled/full Check writers
remain available. Bootstrap saving remains owned by the existing runtime
seed. This native job builds the addon without running the package test
campaign, and it does not add a dependency to any PR gate.

Every addon invocation still creates a fresh `VIZE_NATIVE_BUILD_TOKEN` and
records its actual Cargo build and HEAD/source identity. `vize_vitrine` must
therefore rebuild even when all its dependencies are reusable. A current
trusted seed is expected to remove the other unchanged workspace rebuilds;
changes to their inputs must continue to invalidate them.

The observed PR setup took 24–26 seconds, so roughly 1–3 minutes is an initial
estimate for this asynchronous main job. Its shared setup additionally installs
the full frozen dependency graph and MoonBit, and a cold cache may take longer.
The seed's duration and the next candidate's fresh-artifact count still need
hosted measurement. This change alone does not establish the five-minute PR
target or explain runner-to-runner build variation.
