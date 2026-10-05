# Cold CLI native graph and profile evidence

Issue: [#7698](https://github.com/ubugeeei-prod/vize/issues/7698).
The [delivered five-layer receipt](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5977962697)
and [existing native phase protocol](./2026-10-04-typechecker-native-phases.md)
remain separate historical evidence.

## Decision and scope

Prepare one benchmark-only follow-up after the five performance layers actually
merged. Collect the current CLI's complete native config/member bytes and
untimed CPU/allocation profile files before choosing another production change.
The source preparation began at actual `76f93fe1e3` and preserves subsequent
main promotion/version/lock changes at `caee165692`. Source Actions qualify the final PR head;
manual evidence must identify its separately pinned main production source.
This document reports no new native measurement or profiling result.

The original target remains a complete fresh-process `vize check` over 500
unique SFCs (2,050,350 bytes), with identical semantic options and diagnostics.
The original controlled max median was 425.5 ms at production `da66dc241c`
([run 37169051230](https://github.com/ubugeeei-prod/vize/actions/runs/37169051230));
42.55 ms is the 10x target. Refreshed whole-command baselines around 434 ms
remain hundreds of milliseconds. Imported/shared-leaf gains, warm projections
and persistent-session startup exclusions cannot replace this target.
Demonstrated 10x and the generated500 default-width improvement remain unfinished.

No production, public API, provider, scheduler, semantic cache, compiler option,
fixture or instruction-ceiling change belongs to this slice. All existing
100+4 ceilings and the neutral corpus/protocol hashes remain unchanged.

## Driver, source and build authority

PR execution preserves the exact PR-head driver/source and the merge base of
freshly fetched `origin/main` with that head. Record the immediate PR base
separately; a stacked parent is not a substitute for main authority. Production
changes in a future PR remain observable as `productionMatchesBaseline=false`;
this observation workflow does not reject all such PRs.

Manual execution accepts only one input: a lowercase immutable 40-character
`source_sha`. The workflow must run from `refs/heads/main` in this repository.
Its exact `github.workflow_sha` is the driver head. Driver and source must both
be ancestors of the freshly fetched, recorded `origin/main`; branch/tag names,
off-main source/driver commits and an environment/ref mismatch are rejected.
This support becomes usable only after the driver actually merges into main.
A release publication hold separately controls when an owned run may start.

The source has a separate clean checkout outside the driver root. Build both
production CLI and projection example from that exact source, using the current
locked `ci-opt` settings (release inheritance, thin LTO, 16 codegen units,
nonincremental build, symbol stripping for Vize). Record source/driver/main and
baseline commit/tree OIDs, run/attempt, workflow identity, complete raw Git
modes/blob deltas and exact bridge hashes before install/build and after replay.
Tracked/staged edits and nonignored untracked inputs, including Cargo's
implicitly discovered `build.rs`, are rejected. Ignored dependency/build outputs
are allowed; fresh pinned checkouts and locked installation remain part of trust.
Reports and duplicated trees live outside both source checkouts.

The existing 12-path infrastructure baseline contract remains an explicit
closed set. Eight named graph/profile/custody helper/test paths, one named
test-only corpus fixture and two companion documents extend this slice to 23
allowed paths; no wildcard
admits a newly named tuning script. Allowed deltas must use regular Git file
modes. Manual source/driver Cargo.toml, Cargo.lock, package.json, pnpm-lock.yaml
and all three neutral generator/protocol files must match byte-for-byte.
Unreviewed production/dependency/corpus drift fails before installation/build.
Actions, runtime dependencies and production build flags retain their pins.

## Actual native graph custody

Every actual CLI shard retains its original argv, cwd, checker count, raw
config bytes/hash, ordered files/include/exclude/references and compiler options.
The pinned npm native runtime must be 7.0.2; retain its executable path, version,
SHA-256 and filesystem identity. Its reported `--listFilesOnly` order, duplicates,
raw stdout/stderr and status stay authoritative. Save every observed member's
exact bytes as content-addressed objects, retaining raw reported/access paths,
canonical path, ordered symlink targets, size, mode, device/inode and nanosecond
mtime/ctime before and after replay. Atime changes caused by reads are excluded.
Concurrent rows publish completed immutable objects atomically in one shared
store; only source object storage is deduplicated, never member/root vectors.

Native member bytes include whichever generated files, declaration/default libs
and transitive package sources the native runtime actually reports. They are not
inferred from authored CLI JSON metadata. The historical 168 captures are
**authored CLI input/config metadata**, not native/transitive graph captures or
seven retained configs. Earlier 1,256/608/108 graph counts describe their old
source only; no count transfers to current main. The first fresh post-Stack
native graph, overlap counts and native phase attribution remain unmeasured.
`graphClosureClaimed=false`: a successful observed listing is not proof of every
resolution/config/package input consulted by the compiler.

## Untimed profile replay and closed relocation

PR runs keep profiling disabled. A qualified main manual run additionally
profiles separate duplicated authored trees, including the clean, minimal-error
and full-corpus-error populations in each existing max/1T mode. The copy preserves
symlinks verbatim. Original and duplicate authored manifests must match before
and after. Every duplicate direct/wrapped pair retains actual raw CLI status,
stdout/stderr and the complete ordered normalized product report. Saved virtual
TS/helpers and full generated code/maps/subspans/features/semantic links must
match their original reference, with before/after generated projections equal.
No projection receipt substitutes for dependency filesystem topology.

Actual production native argv must be exactly `--checkers 1 --pretty false
--project <live config>`. Append only `--pprofDir <new unique per-run/shard dir>`.
Retain both original and replay raw Buffer streams. Exit status and stderr must
match exactly. Replay stdout must equal original stdout plus exactly the native
7.0.2 terminal Memory/CPU path lines for the actual spawned PID's two expected
files. No generic line removal can hide a diagnostic. Unexpected/missing/reused
files, symlink/hardlink profiles, truncation, runtime drift, invalid/empty gzip
and any extra or reordered path line fail with a retained partial receipt.
The pinned implementation is [native pprof.go](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/pprof/pprof.go#L22).

Canon hashes the canonical authored root into a sibling external cache namespace
([identity source](https://github.com/ubugeeei-prod/vize/blob/76f93fe1e3c46ac8c86157a520302e26c50ac871/crates/vize_canon/src/batch/virtual_project/identity.rs#L27));
`.vize/canon/projects` is legacy cleanup-only. Duplicate comparison verifies that
exact key and the same parent storage, raw config bytes/options and native member
order. Only the corresponding authored non-node_modules roots and proven virtual
roots may relocate. All other canonical paths, link targets and revisions,
including external libraries, stay exact. Each graph archive hash and its summary
must agree. Ancestor package lookup or config rebasing can change after a copy;
unknown topology/identity/config differences fail closed. This bounded comparison
is not a general proof of portable project or resolver equivalence.

Profile gzip validation certifies only the compressed container. Semantic pprof
protobuf decoding, CPU/allocation sample totals and phase attribution are **not
implemented** and remain unknown/null in the report. These files may guide a
later decoder/proposal after successful actual replay; they provide no measured
cache gain, startup attribution or quantified bottleneck conclusion by themselves.

## Warm states, freshness and remaining work

Graph listing/hashing deliberately warms files before native command/phase
replays. Shards may overlap and contend with other original/replayed checks.
All profile/native phase/child-wall observations are instrumentation-only; they
are excluded from speed comparisons. Filesystem caches are not evicted. Keep
fresh-process first use, filesystem-warm fresh CLI, disk-semantic-cache CLI
(unimplemented), and live persistent session (unmeasured) distinct. Startup and
Program construction stay null; do not subtract separate measurements or sum
parallel work into end-to-end wall time.

Preserve every ordered native diagnostic, UTF-16 source mapping, message/category,
owner, global/options/syntax/semantic phase gate and full product status. Existing
freshness controls create an error, repair it with equal length and restored
nanosecond mtime, then delete it, each against a new direct CLI. Generated code,
all maps/links, inputs, helpers, configs, members and binaries must stay unchanged.
No successful fallback substitutes for failed native evidence.

A seven-Program single-host API timing arm remains **ineligible**. Pinned 7.0.2
has a session parse cache and file-local binder reuse, but creates independent
Program checker pools. The available granular API requires exact diagnostics,
phase and ordering proof; current API checker ownership/scheduling differs from
CLI `--checkers 1`, and coarse project/file/snapshot endpoints are absent. This
slice neither dispatches that arm nor changes provider ownership. See the
[pinned parse cache](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/project/parsecache.go#L10)
and [Program construction](https://github.com/microsoft/typescript-go/blob/2bd066d87f5bafd315be9f40889d0a60b9e58e0b/internal/compiler/program.go#L285).

TODO: after final exact-source Actions, actual main merge and explicit publication
thaw, run this qualified main driver with immutable source custody. Audit complete
native configs/root orders/member bytes/raw vectors and bounded relocation first.
Then decode profiles semantically before reporting sample counts or attribution.
Only concrete evidence can justify a new production program/cache experiment;
that requires another paired #7698/docs decision and whole-command cold target,
complete ordered diagnostics, revision/context safety and unchanged budget gates.

## First source Actions warning correction

Source `9e07bfe280` failed the zero-warning JS gate in
[Check 37192239846, job 111406695262](https://github.com/ubugeeei-prod/vize/actions/runs/37192239846/job/111406695262).
Formatting all 7,477 files passed; lint over 6,088 files found 13
`typescript/no-floating-promises` warnings in the new custody test's top-level
registrations. The earlier focused absolute-path check ran from a different,
dependency-equipped checkout and missed contextual Node test return types;
its zero warnings did not establish whole-workspace CI warning parity.

Await the 13 top-level registrations without changing any of the 40 laws,
assertions, source/allowlist/production guards or the zero-warning budget.
Verify the affected 40 controls and configured lint inside this exact isolated
checkout, then require fresh exact-successor source Actions. Every collector,
profile, production, lock and neutral input blob remains unchanged. This is a
test/documentation correction, not a runtime or policy change.

[Native phase run 37192239409](https://github.com/ubugeeei-prod/vize/actions/runs/37192239409)
passed at `9e07bfe280`; its first post-Stack raw observations belong to that source.
A test-only successor can prove unchanged collector/production blobs, but cannot
transfer that run's head qualification. Final source checks remain required;
manual profiling, semantic decoding, Ready and queue admission stay unqualified.

## Verified manual dependency link after the failed first campaign

The [first actual-main manual campaign 37195795568](https://github.com/ubugeeei-prod/vize/actions/runs/37195795568)
failed before projection Clippy, production build or native capture. Its retained
pre-install custody proves workflow, driver, source and baseline were all immutable
`bcf3e024bde363a4d9e3cc2e7745af771cd5b51f`; all seven bridge hashes matched and all
98 synthetic laws passed. After the locked install, the workflow intentionally
created `sourceRoot/node_modules` pointing to `driverRoot/node_modules`. The source
`.gitignore` rule `node_modules/` matches a directory but leaves that symlink
untracked, so the next strict custody gate rejected `node_modules`.

[Failed artifact 11301276123](https://github.com/ubugeeei-prod/vize/actions/runs/37195795568/artifacts/11301276123),
ZIP SHA-256 `04202ba866579bcd89e9211401f0f614e4bcaeea0258fb3835817f066e234876`,
contains only successful pre-install custody. No newer failure receipt, native
graph, full product report, code/map projection or CPU/allocation profile was
produced. Duplicate-tree equivalence was never evaluated. Preserve this failed
campaign separately; its 98 laws establish no actual profile or speed result.

After exact manual context, commits, fetched-main ancestry, production delta and
all seven bridge checks pass, verify only the literal source dependency link.
Its raw target must equal the canonical absolute `driverRoot/node_modules`; the
driver target must be a physical directory, opened with directory/no-follow
flags and matched by descriptor identity. Retain canonical target and the complete
symlink device/inode/mode/size/mtime/ctime revision, fenced before and after the
read. Record target directory device/inode/mode; cache namespace changes do not
pretend to be content changes. This proves directory identity, not recursive
immutability of installed dependencies: native runtime/member byte and revision
checks remain required at capture.

Use NUL-delimited Git paths. Only that verified manual source `node_modules`
token can be admitted; inspect its target even if an ignore rule hides it.
An existing manual driver root must be physical even before source-link setup.
Relative, offsite, dangling, cyclic or driver-side links, retargeting/replacement
and every other untracked input remain rejected. PR custody and existing ignored
physical-directory handling stay the same. Pre-install absence may transition to
the verified install link; the benchmark's post-install initial and final custody
must match exactly. Do not change `.gitignore` or bypass the untracked guard.

The original twelve infrastructure paths stay byte-for-byte in their closed list.
One dedicated dependency-link test path expands the explicit additions from eleven
to twelve (24 total), with its driver-provenance hash pinned. Keep the existing
40 laws, all assertion bodies and the zero-warning policy; only the explicit
allowlist-count expectations change. Add real filesystem/Git integration controls
for install linkage, hidden bad targets, replacement and unrelated input paths.
Local 36 new controls plus the original 40 laws pass (76/76, no skips/failures);
configured formatting and type-aware lint pass with zero warnings. These include
real in-call symlink-revision and directory-replacement fences, without a native run.

The [paired #7698 decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5979312568)
retains the failure and this narrow repair; local controls do not qualify a new
source head or an actual profiling campaign.

Require root review of this narrow repair, fresh exact-source Actions, all protected
queue gates and actual signed merge before proposing another actual-main campaign.
No automatic redispatch, production/API/provider/cache/scheduler/flag/corpus/map
normalizer or instruction-ceiling change follows from this failure. CPU/allocation
profiles, semantic attribution, startup/Program ownership and 10x remain unmeasured.

## Reviewed temporary semantic decoder

The offline proposal uses pinned official `google/pprof` commit
`4902fdda35c867f2c3e11bc3881d7d2f7d4f2bfa`, copied byte-exact from an existing
cached tool: no package install, native probe, external symbols or repository Go
adoption. The concrete review manifest is SHA-256
`a709333c0d17a57e823a4748d7443ecf3b5fcc41e282151e06f57d7523d5978a`;
official source/copy manifest `fc5d9b57344c84697dea0ee64000cd950f5364cc92c0890c000448d26a535f7a`,
decoder binary `2b10e1b83f9786bd0677802306c8088dfe13d9a90e5f7a22eaeee4c3c517f348`,
and final controls `6ba2f5e52a936f1c4d6eeafe3ffe1618a347bbbde70d9f3e657e53f67de6974b`.
Root review independently verifies all 41 leaf controls / 45 Go PASS records,
zero failures/skips, source hashes and terminal package PASS; rejected initial
controls remain separate. This approves temporary offline processing only.

Bound one gzip member and verify CRC/trailing bytes before exported
`ParseUncompressed` plus `CheckValid`; preserve raw path/hash and failed receipts.
Paired controls show the official parser accepts an unknown nonzero mapping ID
after converting it to nil and supplies an empty PeriodType when absent.
Supplementary bounded wire-reference checks and observed field presence refuse
the former and preserve absence in the latter. Literal mapping filenames, raw
string/label/table/inline stack order, 64-bit values and separate sample dimensions
remain authoritative; flat plus unresolved totals reconcile independently, while
cumulative overlapping functions are never added. Phase, startup, Program,
sample adequacy and speed remain unknown; weights are not wall time or peak RSS.

No actual profile has been processed. Before interpretation, require the repaired
helper's actual protected merge, a separately approved fresh source=driver campaign,
and reviewed original raw profile, executable, runtime and source joins. Local
producer metadata cannot establish the CI producer's source correspondence. The
[paired #7698 decision](https://github.com/ubugeeei-prod/vize/issues/7698#issuecomment-5979648443)
retains this review; no profile/container or cold whole-CLI gain follows from
synthetic decoder controls. The unchanged generated500 425.5→42.55 ms goal remains open.

## Current-main integration of the independent dependency-link repair

The existing [#7810](https://github.com/ubugeeei-prod/vize/pull/7810) integrates
actual main `40e80dede2cce89afad475c713e521428dd1b333`, preserving its two original
authored commits and verified reporter trailers. All four dependency-link,
source-custody, custody-test and phase-driver code/test blobs remain byte-exact
at reviewed source `8778f93ea9fd4ae7d54cfee68fa328272ecdc9a2`; every incoming
production, workflow, fixture and instruction-budget change is retained.

The historical exact-source Check `37200665418` and native phase run
`37200665090` remain evidence for `8778f93`, not the new integrated head.
Require fresh exact-head Actions, all 134 native controls and complete ordered
CLI/code/map/diagnostic/freshness observations, then protected full Rust and
100+4 instruction gates and actual signed merge. The repurposed #7857 no longer
requires this runtime parent; detach its old native Stack relation and verify
#7810 has no Stack before independent auto-merge. This integration authorizes
no manual profile campaign or ranking claim. The original failed pilot,
offline synthetic decoder and unmeasured allocation/CPU/attribution/10x limits
above remain unchanged; recursive installed-dependency immutability is unclaimed.
