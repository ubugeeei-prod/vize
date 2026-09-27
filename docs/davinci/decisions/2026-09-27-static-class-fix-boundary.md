# Static-class autofixes preserve the tag boundary

Issue: [#6920](https://github.com/ubugeeei-prod/vize/issues/6920).
History preparation: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).

## Decision

Template/SFC binding ranges are end-exclusive and already include the closing
attribute quote. `vapor/prefer-static-class` must replace exactly that range;
adding one to its end consumes a tag delimiter or the whitespace before the
next attribute. Use the existing end unchanged in `report_static_class`.
The diagnostic range, message, severity, labels and rule selection stay as
before. The offer's edit end and valid applied source are the behavior repair.

The actual pre-fix SFC offer was `31..55`, where the binding is `31..54`.
It produced `<div class="static-class"</div>` and three parser errors. The
existing bare-template Debug test recorded an equally excessive end (`29`
instead of `28`) but did not apply the fix and inspect the resulting document.
The complete public API history observer exposed that missing contract.

Nine real corpus inputs at
`tests/_fixtures/differential/lint-static-class/cases.json` independently pin
exact offered spans and whole applied bytes. The integration test uses the
actual public template/SFC entry points, applies each genuine offer to the
original bytes, and requires zero diagnostics after re-lint. It keeps no-fix
controls for static class collisions, dynamic expressions and empty input.
Full result snapshots preserve all initial/subsequent diagnostic fields.

## Historical evidence

The original 13-case JSON is retained byte-exact as
`before-fix-history-cases.json`. Its original bad static-class snapshot and
source/binary capture receipt are unchanged. That original observation was
marked **unsatisfied**, not accepted as a repaired requirement. The current
history runner uses a separately named corrected variant for the same input,
still executes 13 cases and retains zero duplicate/native/whole-history credit.

The parent capture documents source-built local observations at its actual
pre-fix source; it does not describe the corrected source or future Actions
build. The static-class boundary is repaired only after actual validation of
this child. Other CSS, Musea and opinionated requirements from the compound
historical fix remain pending. #6881 stays open.

## Validation and publication

Actual locked/offline source-built corpus/history tests passed: nine corpus
inputs with six genuine fixes, plus 13 current history cases. Separate executions
with snapshot updates disabled matched every golden. The five original rule
tests also passed; their existing Debug oracle changes only the offered edit
end from `29` to `28`. All original 13 history goldens, old receipt and archived
input JSON retain their original SHA256 values. Strict targeted Clippy with
warnings denied and `cargo fmt --all --check` passed.
The [corrected capture receipt](./2026-09-27-static-class-fix-boundary-receipt.json)
binds source, Rust/Cargo profile/features, both real test executables and raw
actual build/execution logs. No cold/full-workspace local suite was run.

Before merge, run a fresh Actions build and the full
merge queue, including the markup differential feature lane. No native linter
adapter, additional pipeline stage, dependency or serialization is introduced.

## Existing inventory shard

The first source-built Actions run passed the public nine-case regression
and the corrected 13-case history test. Required tooling then detected the
new test import in the existing linter migration surface shard. The official
generator adds one `test/dev` L0 row for `static_class_fix.rs`; all 19
existing artifact files pass its byte-exact check. This bounded shard
refresh follows the observer parent without changing its raw capture
identities. Separate whole-repository ledger and T0 cleanup remains pending.
