This is step 1b of the #6830 redesign record (#13396). The maintainer decided to cut jobs per PR and reduce path-scoped side workflows on PRs; whatever leaves PRs must still run in the merge queue or nightly.

The workflows below were the most frequent `pull_request` runs other than `Check` (repo runs API, 1,000 most recent PR-event runs since 2026-11-18 08:00 UTC):

| workflow | PR runs | PR branches | summed wall min | where it still runs |
|---|---|---|---|---|
| `zizmor` | 66 | 33 | 359 | PRs that change `.github/**`; main pushes to `.github/**`, releases and weekly (unchanged) |
| `musea-browser` | 66 | 33 | 618 | main per commit and nightly (unchanged) |
| `nuxt-style-build` | 65 | 33 | 294 | main per commit and nightly, and in merge candidates via `nuxt-vite-release-products` (unchanged) |
| `content-mapper-instance-globals` | 56 | 28 | 1,311 | nightly Full qualification: its campaign test is in the merge-tier tooling plan, pinned by a test; dispatch remains |
| `v1-supported-matrix` | 52 | 25 | 236 | nightly Full qualification calls it (unchanged) |
| `volar-plugin` | 51 | 25 | 314 | new nightly schedule `7 4 * * *` |
| `canon-ecosystem-producer` | 50 | 24 | 287 | new nightly schedule `19 4 * * *` |
| `nuxt3-module-build` | 50 | 25 | 368 | merge candidates call it, plus main per commit (unchanged); merge and main runs carry every PR campaign, pinned by a test |

In total these were 456 PR-triggered workflow runs (3,787 wall min) in that window, which is about 14 runs for each of the 33 PR branches in the window. `zizmor` follows the original #6830 decision (only PRs touching `.github/**`).

These stay on PRs for now:

- `jsx-authored-native-modules` and `parcel-transformer` have PR-specific plan jobs and path-regression tests.
- `pr-performance-observation` is the fleet's required measurement.

No required check changes; none of these is a required context.

## Tests

The tests that pinned PR triggers now pin where the coverage moved:

- `content-mapper-instance-globals-workflow`: dispatch-only, and the campaign is in the merge tier but not in a PR plan.
- `nuxt-module-ci-scope`, `nuxt5-runtime` and `nuxt5-dev-runtime`: no PR trigger; the push paths keep every input; merge and push runs include all three PR campaigns.
- `github-workflows-zizmor`: the PR trigger is scoped to `.github/**`.

Locally, 80 tooling test files that read these workflows or the tooling planner: 353 pass. Two kinds of failure don't depend on this change:

- 11 `setup-wild-http-recovery` cases need GNU coreutils;
- 1 `docs-stability` case needs `cargo metadata` outside nix.

The remaining failures were the trigger pins above, fixed in this PR; that suite now passes 49/49. I reverted the jsx/parcel moves after their pins showed PR-specific logic.

## After

Pending: PR-triggered side workflow runs per PR head after merge, and Check queue wait, posted on #6830.

## Product binaries

Unchanged. Only workflow triggers, tests and a planner comment change.

🤖 Generated with [Claude Code](https://claude.com/claude-code)
