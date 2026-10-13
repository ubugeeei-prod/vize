# Complete original input reads in the production reach sweep

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

## Decision

Use the existing shared `read_corpus_source` for the single production reach
sweep read. It resolves each logical path component before reading the entire
original UTF-8 source. Retain every selected logical input and existing compiler
shape, parity comparison, reach floor, and strict zero-unreadable requirement.
If an original source cannot be resolved or decoded, fail with its complete
logical path, original IO message, error kind, and OS error. No input is omitted.

## Observed failure and limits of attribution

At source `b32ae8b120501bcb166c91b5a9d1e4112b5cf294`, required
[Check 38151468856](https://github.com/ubugeeei-prod/vize/actions/runs/38151468856)
job `114506225754` passed all 18 production reach tests but its evidence validator
refused canonical `unreadable=1` among 44,367 selected files. The caller used a
whole-path `fs::read_to_string` and discarded the failing path and IO error.
The committed scope measured 502 files with zero unreadable inputs.

The authenticated reach artifact `11707002645` has SHA-256
`81348e08bd8d058e4d5831a43a31ee733cfb18a2d32ce0841cdcd5b778df3d52`.
Compared with successful [Check 38151804577](https://github.com/ubugeeei-prod/vize/actions/runs/38151804577)
source `3407da4f4efa502f82624c483938af857ccba07f`, 44,333 complete
path/length/SHA-256/Git-blob tuples are identical. Only the pinned npmx.dev
revision changes the remaining tuples; all 224 old npmx Vue bodies were
authenticated and decode as UTF-8. The new committed directive fixture is not
in the canonical vector. The same source's DOM job `114506225720` and SSR job
`114506225712` read all 44,367 selected inputs through the shared strict reader.

These receipts establish the inconsistent read caller. The old aggregate-only
failure does not establish its exact original path or OS error. A valid ancestor
alias chain exceeding the single kernel lookup limit is a durable regression
control, not an attribution of the old unreadable row. Fresh full original
corpus evidence must qualify the correction before merge.

## Durable controls

The reach caller's own Unix law creates a real ancestor alias chain of 80
components. A direct whole-path read fails with the original `FilesystemLoop`
OS error, while the complete original BOM, Japanese text, emoji, and CRLF bytes
remain intact. The production sweep must account for the selected input in all
four shapes, with zero unreadable or omitted templates and no parity failures.

Two negative controls exercise the same sweep: a genuinely self-referential
source link and invalid UTF-8. Both must fail with the whole logical path and
original IO/error-kind/OS-error payload. Matching only a prefix or returning an
unreadable counter does not satisfy these laws.

## Verification still required

Use exact-head Actions for the caller laws, every original canonical observer,
the unchanged corpus input vector and source bytes, then the protected merge
queue's full suites and actual merge. Existing root attribute-name Fuzz repair
`130331f` remains in the fresh main-based source. No production compiler source,
native transport policy, dependencies, locks, budgets, goldens, or workflow gate
changes are part of this slice. Publication and broader Matrix parity remain
separate acceptance work.
