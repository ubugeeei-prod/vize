# CSS global ownership on the native type-aware route

Issue: [#7976](https://github.com/ubugeeei-prod/vize/issues/7976).

## Decision

The native type-aware driver owns the existing parsed template until every
script and type diagnostic has been collected. Append the existing CSS pass
there, before releasing that root, and remove the caller's rootless CSS append.
Every existing early return and the ordinary completion use the same finalizer.
CSS therefore runs exactly once after script/type diagnostics, including
scriptless, empty-script, static-warning and empty-query exits.

The CSS ownership input is the same already parsed root, with no additional
template parse, serialized intermediate or pipeline stage. Fatal parses supply
no root. External `src` templates and non-HTML template languages also supply no
ownership evidence; their existing conservative CSS findings remain unchanged.
An absent template retains the same conservative behavior. Native type severity
application still happens after the driver; CSS retains its existing rule
override and count accounting.

## Durable evidence

`crates/vize_patina/tests/fixtures/issue-7976-type-aware/cases.json` independently
authors 16 complete native-route diagnostic packets. The test explicitly enables
`type/require-typed-props` together with `css/no-display-none`, and a unit law pins
that this selection activates the actual native driver. An intentionally missing
Corsa path makes accidental runtime queries visible instead of silently skipping
the regressions.

The packets cover foreign/local global subjects in scripted and scriptless
components, empty scripts, query-free collector completion, missing/external/
non-HTML/fatal templates, explicitly HTML templates, multiple styles, original
deep/slotted exemptions, and a complete script-error → type-warning → CSS-warning
order. Whole diagnostic fields and error/warning counts are checked repeatedly
with normal CSS severity, CSS promoted to error, and CSS disabled. The original
54-case external-target, 66-case global and 62-case compound corpora are retained
byte for byte; their hashes are recorded in the companion `source.json`.

## Qualification

This is source preparation. The root delivery task owns the paired issue comment
and canonical record clause, fresh exact-head source/whole-history/API/CLI
Actions, protected queue qualification, actual merge and installed release
replay. Historical source greens grant no current delivery or publication credit.
No upstream comment or change is part of this work.

The consumer migration inventory is regenerated from the current owning source.
Only the Patina shard changes: the three existing driver import rows move from
line 22 to line 23, one production Relief-root row records `driver/css.rs`, and
one L0 test/dev row records `css_type_aware_ownership.rs`. The full generator
check passes; no unrelated producer or consumer row is changed.

Severity review confirms CSS is overridden once by its existing append helper,
which recounts the whole result. The outer native severity pass selects only
`type/*` diagnostics, so it does not reapply CSS overrides. A changed type
severity triggers another whole-result recount; an already matching severity
does nothing. The existing complete-result checks pin both CSS warning/error
counts and the ordered script/type/CSS packets from independently authored
constructor inputs, without captured product snapshots.

## Canonical clause for root integration

[#7976 native type-aware CSS ownership](./2026-10-08-css-type-aware-global-ownership.md)
reuses the driver's already parsed template before it is dropped; every existing
early exit and completion appends CSS exactly once after all script/type findings,
with no additional parse or pipeline stage. Fatal, absent, external and non-HTML
template roots remain conservative. Sixteen independently authored complete
native-route packets pin actual type-rule activation without Corsa, global
foreign/local controls, script/type/CSS order, repeated counts, severity and CSS
disable behavior while all original CSS corpus bytes remain unchanged. This is
source preparation; fresh exact-head whole-history/API/CLI Actions, protected
queue acceptance, actual merge and supported installed-release replay remain
root-owned requirements before closure or publication credit.

## First source qualification correction

Exact source `e5652013` compiles the affected Rust tests, but
[Check 37730888883](https://github.com/ubugeeei-prod/vize/actions/runs/37730888883)
rejects the stale Croquis census in tooling shards 1 and 2. The shared
`SfcDescriptor` helper signature adds one genuine occurrence: only the Patina
census shard changes from 16/25 to 17/26. Regenerate with the actual
`croquis-consumers.mjs` producer, retaining all other 19 files, product sources,
fixture bytes, complete expectations and gates. The generator check passes
all 20 artifacts. Preserve the failed source record; this inventory-only
correction still requires fresh successor Actions and protected qualification.
