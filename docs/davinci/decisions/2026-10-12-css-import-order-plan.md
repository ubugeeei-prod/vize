# CSS import order: bounded qualification plan

Related issues: [#3864](https://github.com/ubugeeei-prod/vize/issues/3864),
[#3295](https://github.com/ubugeeei-prod/vize/issues/3295).
Parent [dependency candidate](./2026-10-12-css-engine-vendor-exit.md).

## Observed failure

On source `b2205cf5a53ac57c4ad3c0a5d92cb649087fe402`, actual checkout
`da002e6f1068181afbf6e8472530bc628730ee75`,
[Actions 38187677652](https://github.com/ubugeeei-prod/vize/actions/runs/38187677652)
fails the unchanged original `bundler::tests::test_bundle` in debug: authored
`print`, then `screen` imports produce `@media screen, print` instead of the
complete existing `@media print, screen` reference. The original input, full
test module and red output are retained. No retry or output normalization
establishes acceptance.

Parallel loading merges duplicate-path media/supports conditions under a mutex
in worker arrival order. The mutex protects memory access; it does not preserve
authored order. `MediaList::or` appends previously unseen conditions.

## Proposed private data flow

This is a read-only design proposal. Safe legacy equivalence remains unproven;
no production delta or expected-output replacement is approved by this record.

1. Retain parallel physical file reservation, resolve, read and parse in phase1.
   Retain the actual source handles, parser options and source-map registrations.
2. In the existing phase2, replay retained import rules and rewritten CSS-module
   references in authored order. Existing AST nodes carry the payload; avoid a
   second duplicated queue of import conditions.
3. Initialize each node from its first logical incoming context, propagate that
   context to descendants once, and fold later duplicate conditions using the
   original OR, empty-media absorption and layer equations.
4. Prefix layer statements once from the first context. Preserve local error
   locations and filenames. Keep the existing CSS-modules-first/import-last
   emission ownership walk and inlining separate from imports-first discovery.

Sorting condition names would destroy a reversed authored `screen`, then `print`
reference. A global one-worker setting would hide the race and remove parallel
loading. Neither is the proposed fix; no pipeline phase is added.

## Before evidence required

The old loader propagates only its first incoming context into descendants.
Rebuilding descendants from the final aggregate parent union changes semantics
beyond the observed ordering defect. Conditional duplicate-parent and diamond
graphs need whole before observations before choosing their behavior.

Moving condition validation after file loading can alter competing semantic,
resolver and parser error priority. Record every full error and location rather
than silently accepting a new winner. Delayed multi-file source-map indices,
source/content/comment ordering and mixed CSS-module references also lack a
complete before authority. Preserving their code alone does not prove equality.

Ordinary self-import and A↔B import revisits can terminate because inlining takes
the rule vector before recursion. They are unqualified controls, not established
crashes. A direct root first self-composes dependency is a separate source-only
recursion candidate: retained CSS-module edges and default parent0/dep0 may
re-enter. It requires a bounded before probe and has no acceptance credit.

## Focused qualification

Preserve all original bundler tests and whole media/supports/nested-layer,
inherited-media and conditional failure references. Add full reversed media,
both unconditional absorption orders, nested/diamond/cycle, CSS-module,
external-import failure and genuine map controls without changing old strings.

Use distinct import spellings that resolve to the same physical path. For pools
8 and32, the first-authored resolve waits until the later-authored provider
actually reads the file, after reservation and condition seeding. Retain the
complete provider event trace and a short process timeout. A sleep or a signal
at resolve return does not prove inverted seeding. Pool1 uses a nonwaiting
sequential protocol and must never wait for another worker.

Run complete debug/release library, original27 CLI plus whole-conflict, source-map,
archive/refusal and hue contracts, original Rust99/Atelier66+6/CLI90 and full
source/protected suites with unchanged instruction limits. Verify full outputs
before timing the same zero-delay fanout, chain, duplicate/diamond, CSS-module
and real-project workloads on the same runner/compiler/provider. Retain every
cold/warm sample; no speed or allocation improvement is established yet.

The next bounded slice is additive before-capture against the existing checked
0.8 candidate provider. It must resolve these named gaps before a reviewed
production repair, main admission, nominal API choice or publication.
