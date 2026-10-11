# Mounted Rust target seed reuse

Tracked in [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Observed cause

The successful main Check at `b7ec71ecd3981fdeffb5df816bd0e811e175b61e`
([run 38102870716](https://github.com/ubugeeei-prod/vize/actions/runs/38102870716))
mounts stable Blacksmith Rust targets, then archives them again through
`actions/cache` under a new source-SHA key. A compatible prefix hit with
`lookup-only: true` still registers a save: the pinned action skips its archive
only when the restored key equals the primary key exactly.

| Job                 | Build/check workload                          | Target archive       | Archive post time |
| ------------------- | --------------------------------------------- | -------------------- | ----------------- |
| `check-vize-apps`   | CLI build 39 s; whole job 419 s               | 14,306,215,764 bytes | 350.306 s         |
| `build-js-packages` | declarations/native/UI 139 s; whole job 364 s | 6,419,453,540 bytes  | 152.695 s         |

The first job's target sticky filesystem holds 85.50 GiB and commits in 3.232 s.
The duplicate archives consume 503.001 s of runner time across these two parallel
jobs. This is measured baseline cost, not a measured reduction in Check latency:
the same run spends 1,669 s in full tooling and 1,101 s in JS package tests.

## Decision

For a target that the existing trusted-default-branch policy allows to save,
select a stable Actions seed key only after `mountpoint` confirms that particular
target is actually mounted. Its identity retains repository, OS, architecture,
runner suffix, role and the existing lock/toolchain fingerprint. Primary and
secondary targets retain independent locks, roles and mount probes.

The first miss saves the current validated target before sticky unmount. Later
exact seed hits retain lookup-only behavior and skip compression and upload.
Failed mounts and GitHub-hosted runners retain their per-source-SHA save keys.
PRs, merge groups and nondefault branches remain restore-only; they can restore
the stable seed through the unchanged compatible prefix. The mounted
`test-scripts` exception and oversized-role exclusions remain unchanged.

No Cargo command, feature selection, source receipt, test, gate or runner changes.
Restored build products remain inputs to ordinary current-source Cargo builds,
never validation results. A stable seed can be older than the previous main
commit; a PR or failed sticky mount may rebuild more source code than with a
fresh per-commit archive. Lock/toolchain changes still select another seed, and
ordinary compatible-source fingerprint checks remain Cargo's responsibility.

## Qualification

Backend-child regressions compare original per-SHA and stable-seed execution
across two real fixture commits: the original writes each target twice; mounted
stable keys write it once while both current-source workloads still execute.
They also cover PR restore/rebuild, independently failed mounts, hosted fallback,
lock/toolchain/role isolation and nested post paths.

Exact-head Actions, protected merge and repeated trusted main runs must verify
the real archive omission. First-seed compression remains expected. A measured
Check speedup is unfinished until those hosted results exist; this change alone
does not solve the full-tooling or JS-package critical paths.
