# Scoped union props preserve accessible names

The source-built 533-fixture UI receipt retained in
[the runtime pairing decision](./2026-10-11-ui-vapor-hydration-runtime.md)
shows that `icon` and `icon-button` lose their accessible names in native SSR.
The original mount and hydration assertions expose the same missing props.
Both components declare a base props type intersected with a union of
decorative and accessible variants. Scoped property collection handled the
intersection but omitted its union branches. The product compiler independently
flattened intersections through its existing text type resolver, silently
skipping the nested union. Its runtime props and template bindings therefore
contained only the base properties.

Under [#6100](https://github.com/ubugeeei-prod/vize/issues/6100) and compiler
fix-history [#6880](https://github.com/ubugeeei-prod/vize/issues/6880), collect
the union's public keys during the existing parsed traversal. Preserve their
first-declaration order and authored branch-local optionality. A shared key
combines its same-module type text and is optional when either declaration
is optional, matching the official Vue
[`mergeElements` contract](https://github.com/vuejs/core/blob/v3.6.0-rc.11/packages/compiler-sfc/src/script/resolveType.ts#L389-L418).
Existing intersection and override behavior remains intact; this adds no
parser or pipeline stage.

The existing text resolver also collects parenthesized/direct union members,
combining shared member types and optionality before its existing runtime-prop
AST inference. This restores both compiler prop declarations and script-context
bindings without routing the legacy compiler through a new provider. Plain
intersection/member output retains its existing path. Existing member helpers
move verbatim in a separate move-only commit; the grandfathered 590-line resolver
shrinks before adding this correction.

Each property retains its lexical module. A shared key from two different
modules remains visible but has unknown type and incomplete resolution:
the current single-module textual representation cannot correctly combine
two independently scoped aliases. Never resolve the second alias in the
first module or in the root SFC. An unresolved union branch likewise retains
observed keys and reports incomplete resolution. This slice does not claim
complete cross-module union inference.

The whole corpus packet in
`tests/_fixtures/differential/compiler/scoped-union-props` contains inline
and imported versions of the actual base/intersection/union shape, the
unchanged imported type source, and a plain-interface control. Source hashes
bind all four inputs. Full golden HTML requires an escaped accessible label,
escaped title text, absent Boolean props cast to false, a bare Boolean prop
updated to true, and complete disposal. Native and official Vue execute the
same sources: Vue 3.5.35 and supported rc.11 SSR, plus rc.11 DOM and Vapor
mount/hydration with root identity and reactive updates. Thirty-six cells
retain complete emitted-module hashes and full outputs; warning/error handlers
must remain empty. The five Rust controls cover field order, shared and
branch-local optionality, imported Croquis metadata, lexical alias collisions,
and unresolved branches. Two additional text-resolution controls retain the
nested base/intersection/union and direct union's branch-local required keys,
shared optional constructor union, and an unchanged object-member literal union.

The initial scoped-only source diagnostic
[38111609804](https://github.com/ubugeeei-prod/vize/actions/runs/38111609804)
passes all 28 original/new scoped-world tests and the unchanged before-source
533 UI controls. Its before and after native bindings are explicitly identified
at `2eed646c3293d83bf14d005c02b068e81909867f` and
`63af12fba86b26c9cb27de424391d1e34e7fa882`. Both fail the unchanged whole native
SSR golden with the identical actual HTML:
`<button title="Refresh &lt;panels&gt;&amp;" arialabel="Reload &quot;feed&quot;&amp;"></button>`.
That is genuine negative evidence that scoped metadata alone does not fix the
product, and requires the text-resolver correction above. No fixture, golden,
warning/error assertion, ledger or budget is changed in response.

The official-only local rehearsal passes all eighteen official cells. That
validates the golden contract; it is not evidence that the source-built native
fix passes. Exact before/after native Actions, the complete original 533 UI
controls, the original rc.9 negative oracle, protected checks, actual merge and
release acceptance remain required. Failure ledgers may remove only entries
proven to pass on that exact source; every remaining reason, fixture and budget
is preserved. The five other previously recorded SSR markup assertions require
their separate provider/browser qualification and receive no waiver here.
