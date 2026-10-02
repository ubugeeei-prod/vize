# Registered complete linter history observations

Issues: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881) and
[#6891](https://github.com/ubugeeei-prod/vize/issues/6891).

## Decision

Register all 36 already published complete Patina history cases in the shared
product manifest/result envelope. Their existing authored JSON and immutable
Insta oracles remain in place; the registry pins their exact hashes and rejects
missing, duplicated or unplanned case/target coordinates. Snapshot metadata is
removed only at its explicit frontmatter boundary. Every complete observation
byte remains in the comparison, including its trailing newline.

| Cohort                 | Planned cases | Actual public observable                                                                                                                                   |
| ---------------------- | ------------: | ---------------------------------------------------------------------------------------------------------------------------------------------------------- |
| Current API history    |            13 | Complete Case/LintResult, ordered diagnostics/help/labels/offered edits, independently applied original-source fixes and full re-lint or unchanged requery |
| NextTick arrow history |            10 | Same complete public observations through standalone script and Unicode/CRLF SFC framing                                                                   |
| Report history         |             4 | Complete Case/Report, all ordered results, unchanged requeries, formatted messages and exact JSON/Text/ANSI bytes                                          |
| Static-class repair    |             9 | Existing exact offered edit span, complete applied bytes and corrected public re-lint/control results                                                      |

`lint_history_observer` is an ordinary source-built Rust example. It invokes
the actual public API/options used by the existing integrations and retains
their complete Debug observations. Its strict input schemas fail on unknown
fields; invalid APIs and malformed inputs fail the process. Each observation
repeats within the process, and the adapter executes two independent fresh
processes before accepting the complete immutable reference. It applies each
real offered fix independently to the original source rather than constructing
edits or combining unrelated offers.

The source-built Cargo artifact and contract probe are bound by the shared
[observer receipt](./2026-10-01-product-observer-source-binding.md). Actual raw
stdout/stderr, statuses, signals, process failures, comparisons and the build
receipt are retained under `target/differential/linter-api/`. Pure registry and
rejection laws run in T0; the real source-built observer execution is an explicit
full T1 case, with no case-level skips or relaxed instruction budget.

Unavailable whole-product native linter execution is explicitly `unsupported`
for all 36 planned coordinates and `not-compared`. Native handled/equivalent
and paired comparisons remain zero. The legacy markup hook differential is
a separate narrower check and cannot supply whole-product native provenance.

## Whole-history ledger and remaining work

The historical issue's 499 touch commits / 255 reported fixes has no recorded
source revision or counting method. Preserve that original count. The complete
pinned enumeration at `b4f25fb6511075aa531be80d645bb0db8cc151e0` has 503 nonmerge
touch commits, 258 conventional fix-title candidates, 245 other titles and 48
merge supplements. Those title counts are not a semantic requirement denominator.

The 36 fixtures register selected requirements and controls, including explicitly
changed successor policies. They do not certify every independent branch of
their compound historical commits. Original broken static-class observations
remain immutable history; the active registry uses the separately merged correct
repair contract, rather than accepting malformed applied output as a fix.

TODO: review every scoped delta, non-fix behavior change, merge resolution and
supersession; associate each real requirement with original source/options and
complete executable diagnostics/edit/application evidence; add missing public
observers and fixtures; retain fresh source-built Actions and protected queue
proof at the exact published heads. Full-history closure and a native adapter
remain unfinished. #6881 stays open and the production linter route is unchanged.
