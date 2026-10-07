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

The spike reports approximately 11 seconds of Vize overhead in editor-ui and
one native lint call per rule/file. Measure the complete custom rule set against
the same source/dependency cohort before claiming a speed-up. n8n's remaining
ESLint parse cost is separate from bridge overhead. Keep exact-head source,
merge-queue and installed-package proof separate, and update the owning issue
when each requirement gains real evidence.
