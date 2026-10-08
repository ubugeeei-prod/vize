# n8n native CLI configuration and entry qualification

Issue: [#8142](https://github.com/ubugeeei-prod/vize/issues/8142).
Prerequisite: [#8260](https://github.com/ubugeeei-prod/vize/pull/8260).

The read-only adoption requirement at n8n
[`5c2a2cf3`](https://github.com/n8n-io/n8n/pull/40393) invokes native
`vize lint` and package-local TypeScript configurations. Its 51 scalar
rules, three independent options, editor warning severity, and six literal
file entries require a CLI producer proof. The earlier Oxlint campaign is
evidence for its frozen contract; it does not qualify this configuration path.

The committed manifest is a Vize-owned functional projection. It records the
requirement revision and config digest, but includes no upstream branch
source or unpublished upstream package artifact. An owned settings package
and freshly built public `vize/config` reproduce the import boundary. Neither
OXC nor Vize Node native bindings are installed in the scratch package.
Upstream n8n remains strictly read-only.

The existing n8n navigation job builds the CLI once, authenticates its exact
source revision, binary digest, path, and version, and runs this bounded
acceptance after the navigation laws. The runner refuses absent or stale
producer receipts. Every failure retains its raw stdout, stderr, command,
status, source configuration, and input identities before assertions. The
always-uploaded artifact also retains independent provider failures.

| Law                   | Required evidence                                                                                                                                                                                                                 |
| --------------------- | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Original selection    | Exact licensed-master Gitlink/tree/blob and physical bytes; all 1369 Vue files in nine roots, including all 19 scriptless files, remain in every complete CLI report.                                                             |
| Scoped settings       | Baseline, effective, repeat, and outside-CWD reports compare whole packets. Only the six selected rule findings may disappear; editor attribute findings change only severity.                                                    |
| Neighbor controls     | Twelve owned sources occupy the six exact entry paths and their adjacent filenames. Positive disabled-rule findings exist before scoping, unrelated findings survive, and neighbors remain diagnosed.                             |
| Independent witnesses | Twelve owned sources plus six option-inversion calls run all 51 explicitly mapped ESLint rules. Complete pinned provider packets, including ranges, fixes, suggestions, and foreign fields, compare against the committed golden. |
| CLI witnesses         | All 18 calls compare the entire public CLI JSON envelope against separate source-derived laws. These laws are not represented as previously captured CLI execution.                                                               |

Seven mappings are explicit: registration maps to `vue/no-undef-components`,
element order maps to `vue/block-order`, and five script rules map to their
official Vue identities. The other 44 identities are literal. A general
implementation-status map must not silently remove a selected requirement.

Source-derived CLI expectations describe the reviewed rule/parser/reporter
code and carry source hashes; only a successful authenticated source run
qualifies them. Timings are individual observations, not performance credit.
The raw complete original reports provide configuration and entry evidence,
not independent semantic parity for every rule. Do not call a passing helper,
projection, or authored subset a full monorepo or installed-release result.

Remaining #8142 gates include complete 51-rule accuracy, the registration-only
casing default, unpublished upstream package resolution, both n8n-local
plugins, retired/type-aware rules, full monorepo lint/typecheck, direct SDK
scriptless coverage, and installed/public-release qualification. Actual
exact-head Actions, protected merge, and publication remain separate terminal
states; attach their immutable evidence to the issue when they finish.

PR #8270 opened with #8260 as its base after the parent entered the protected
queue. GitHub refused native Stack registration for that queued parent. Do
not cancel its running candidate to manufacture Stack membership: keep the
child explicitly unlinked, then rebase and retarget it onto genuinely fresh
main after the parent actually merges, and rerun source Actions. Future
dependent children must be registered before parent queue admission.
