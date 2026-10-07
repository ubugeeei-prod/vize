# n8n adoption requirements (2026-10-07)

Tracking issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).

The maintainer prioritizes n8n compatibility first, completion of every
acceptance criterion in the 61 original P0 delivery issues, sustained LSP and
typechecker performance/correctness/stability, and frequent successful releases.
Work proceeds in isolated `wt` worktrees and independent agent lanes. Dependent
slices use native GitHub Stacks; green ready prefixes and independent PRs enter
the protected merge queue. Track checks, actual merges and public releases
concurrently without leaving PRs unattended. This issue must stay open until
its acceptance requirements are met; fixture registration alone does not close
the adoption umbrella or any original P0 issue.

The upstream [adoption spike](https://github.com/n8n-io/n8n/pull/40393) is still
open at audit revision `aa173be0c65c0646a7fcec32d2c18e1eaacbc8ff`. Upstream is
read-only: no comments, issues, pull requests, pushes or other state changes.

The public submodule pins licensed master revision
`e882e8a483f433facb47bab9b407d0ec00a81172`. Its `LICENSE.md` excludes branches
other than master; enterprise source has separate development/testing terms in
`LICENSE_EE.md`. Preserve both license files and keep the fixture as a gitlink.
The adoption PR is requirement evidence, not the fixture revision or a merged
adoption claim.

`tests/_fixtures/n8n-adoption.json` records the exact 51 Vize rule settings and
options, package warning/override scopes and eight retired rules from the
adoption revision. It also records the actual master corpus: 1,369 Vue SFCs,
including 19 without script, across all nine Vue package roots. This differs
from the spike's reported editor-ui count and 11 scriptless files; do not
transfer those numbers between commits.

| Acceptance requirement         | Required evidence                                                                            | State                                                                     |
| ------------------------------ | -------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------- |
| Complete application fixture   | Pinned gitlink, all `packages/**/*.vue`, exact package/scriptless inventory                  | Added; Actions pending                                                    |
| Custom rules and options       | All 51 rules, casing/order options, editor-ui warning and six scoped overrides               | Frozen; source plugin acceptance pending                                  |
| One native call per file       | Custom rule map batching, override/config/source invalidation, matched timing                | Pending [#8144](https://github.com/ubugeeei-prod/vize/issues/8144)        |
| Accurate diagnostic locations  | Real template coordinates and valid Oxlint script positions; unchanged existing script spans | Pending exact-source adoption replay                                      |
| Scriptless SFC coverage        | Every scriptless input gets template checks without shifting script diagnostics              | Pending exact-source adoption replay                                      |
| TypeScript template unions     | `el as Element \| null` has no Vue 2 filter false positive                                   | Current fix exists; adoption replay pending                               |
| Config inheritance/suppression | Explicit per-package settings, scoped overrides and actual `.mts` discovery                  | Pending [#7903](https://github.com/ubugeeei-prod/vize/issues/7903) replay |
| Type-aware Vue coverage        | Whole SFC script/template diagnostic parity                                                  | Unfinished: spike retains ESLint for type-aware `.vue` rules              |
| Retired rule coverage          | Independent oracles for all eight omissions                                                  | Unfinished; preserve the recorded gap                                     |
| Delivery                       | Exact-head source Actions, protected merge and verified public package                       | Pending                                                                   |

The existing Real Project Matrix automatically includes the new registry row
in compiler, linter, typechecker, formatter, highlighting and LSP lifecycle
shards. The targeted `n8n adoption fixture` Actions job hydrates n8n and fails
on missing or changed corpus. Generic matrix coverage and fixture inventory do
not establish custom plugin parity or a speed improvement. n8n dependency
installation, vue-tsc parity and authored LSP oracles remain follow-up work;
no typechecker performance baseline is claimed by this registration.

The source tooling dependency unblock is [#8148](https://github.com/ubugeeei-prod/vize/pull/8148).
It changes the SDK cohort independently; its success does not qualify the
fixture, bridge, protected merge or installed-package acceptance by itself.

The unchanged n8n corpus exposed nine complete-template differences and two
UserSelect production-module differences at fixture source
`72aac62d7c2932c71075d579c76459b270d909da`; its compiler gate remains failed.
The existing fixture PR retains all original files and the original 16-error
allowlist and ceilings. Repair qualification requires the whole 43,000-plus
template corpus, complete production code/maps/results, and authored runtime
behavior. A matching result from two changed producers alone is insufficient.

The before/after job authenticates an immutable capture-only ancestor against
that original source and a reviewed immutable repair anchor that genuinely
descends from the capture ancestor. The baseline's production and dependency
bytes stay identical; both phases use identical capture drivers, input hashes
and build recipes. The candidate's exact source tree, toolchain and binary hash
remain recorded. Baseline-to-candidate ancestry is reported honestly because
protected squash merges need not retain it; baseline-to-repair-anchor ancestry
must pass. Complete raw packets are saved before parity/runtime assertions, so
a failed gate retains its evidence and receives no completion credit.
The immutable capture-only commit is
`1f7b027ab26e2c031592f3c8546ea6fa7872eeeb`; the reviewed descending repair
anchor is `ac09d4d3624d3e0fe5b1852300d9860af2172ba3`. Preserve their real
commit chain when integrating the existing PR. The ordinary default-mode
baseline has two complete legacy errors with missing locations; retain those
`null` locations as failed historical evidence. Its two native diagnostics
already have byte spans. The official per-prior-branch law expands their full
wire metadata to three without dropping stage, severity, parts, witness or
debug values; current legacy locations independently match the frozen official
UTF-16 positions converted to UTF-8 spans.

The independent reference is n8n's locked Vue/compiler-sfc 3.5.26 browser
bundle, authenticated by SHA256. All ten complete originals retain parse,
script metadata, unbound and real-binding template modes, inline component
modules, maps and diagnostics. The assertion-only judge recomputes complete
code, preamble, maps and metadata with the same pinned API/mode without adding
a production stage or rewriting the retained official packet. Unexpected API
refusals fail qualification; unsupported style preprocessing stays explicitly
unqualified. InstanceAi is accepted in module/prefix mode, while function mode
must retain its three ordered duplicate-key diagnostics and diagnosed code;
that mode receives no successful module/runtime credit. Official UTF-16 offsets
and Vize UTF-8 spans are distinct coordinate systems. The official maps-on
shared-location anomaly remains raw evidence, separate from accurate authored
source locations. Current source before/after, full corpus, protected merge and
public consumer acceptance remain pending.

The spike reports approximately 11 seconds of Vize overhead in editor-ui and
one native lint call per rule/file. Measure the complete custom rule set against
the same source/dependency cohort before claiming a speed-up. n8n's remaining
ESLint parse cost is separate from bridge overhead. Keep exact-head source,
merge-queue and installed-package proof separate, and update the owning issue
when each requirement gains real evidence.
