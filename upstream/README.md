# Upstream sources

This directory owns every externally-derived source tree. Pristine profiling
checkouts remain top-level submodules; Vize-maintained, publishable forks live
under `upstream/forks/` and are the crates the workspace builds.

| Path                  | Pin                                                            | Why this commit                                                                               |
| --------------------- | -------------------------------------------------------------- | --------------------------------------------------------------------------------------------- |
| `upstream/oxc`        | `75efdca4747c4ad9b252542743098d4894f8005b` (`crates_v0.156.0`) | Current fast parser. The workspace builds the maintained fork at `upstream/forks/oxc_parser`. |
| `upstream/tsrs`       | `77d3cddb44540734abc33a42ccca4d3f33bee527` (`v0.20.1`)         | Checker used for the content-mapper wall.                                                     |
| `upstream/typescript` | `3e9236361fa97a9e6c685f4833ac457ae2d8c50a`                     | The TypeScript 7.1 beta commit tsrs 0.20.1 follows.                                           |

Phase times already come from `tsrs --extendedDiagnostics`. Do not add another
timer until that report is missing a phase. `--fast` is the existing opt-in
for the checker modes that differ from the pinned TypeScript defaults.
