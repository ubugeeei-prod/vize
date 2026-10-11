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

The complete actual native source run
[38112877041](https://github.com/ubugeeei-prod/vize/actions/runs/38112877041)
retains the unchanged failing-before golden at source
`2eed646c3293d83bf14d005c02b068e81909867f`, then passes all thirty-six native and
official cells at corrected source `6a279bc77f531c95abb58ee5b90c634354ef5f3f`.
Rust formatting and all 28 scoped-world, 9 text-resolution and 32 runtime-prop
controls pass. Standard and production UI tests each pass all 3,833 tests in
497 files. Every original source/product step succeeds; the final observer
fails because it incorrectly expected unsupported rc.9 mount to improve too.

The corrected read-only receipt
[38113703526](https://github.com/ubugeeei-prod/vize/actions/runs/38113703526)
succeeds over that immutable run's complete raw outputs. It verifies every
original product/source step's actual success and rebuilds the exact corrected
source, requiring the evaluator's native binding SHA to equal the original
corrected binding. The before binary is
`de9d8c41b04db0d100afafe62df91394e33573bfde2cd93fbcf0cec776d716bb`;
the corrected and evaluator binaries are
`b9c5f92d0dc93571431bdd2bfb7f4691d73836c7d9e0054c9e0782750bdd38fe`.
All 533 original fixtures, assertions, source files and failure-ledger bytes
remain in the before/after run. All three plain-control compiler results remain
byte-identical. Every other client problem record and all other 531 complete
SSR HTML outputs remain byte-identical for both provider tracks.

| Original 533-fixture lane | Before passing rows | After passing rows | Proven entries removed |
| --- | ---: | ---: | ---: |
| Supported rc.11 mount | 508 | 510 | 2 |
| Supported rc.11 hydrate | 485 | 487 | 2 |
| Reference rc.9 mount | 471 | 471 | 0 |
| Reference rc.9 hydrate | 369 | 371 | 2 |

Only the corresponding `icon`/`icon-button` entries ratchet down: supported
mount/hydrate now retain 23/46 known entries; reference mount/hydrate retain
60/156. Every other entry and reason is unchanged. The unsupported rc.9 mount
SVG namespace assertions remain failures, including the nested Icon in
IconButton, so both mount records are preserved. The four original ProgressBar
CSS-property exceptions remain required rc.9 negative evidence and never enter
a failure ledger; all four pass on the supported matching rc.11 providers.

Applying every original authored SSR markup assertion to the complete raw
outputs improves supported correctness from 526/533 to 528/533 and reference
correctness from 530/533 to 532/533. Rendering all 533 successfully is a separate
measurement. The five other supported markup gaps recorded in the original
runtime decision remain unresolved and receive no assertion change or waiver.
The [durable source receipt](../evidence/2026-10-11-scoped-union-props-source.json)
retains whole Icon/IconButton SFCs, their original complete fixture module,
original assertion functions, complete before/after HTML, native bindings,
all lane counts, exact ledger removals and the unchanged-control output hashes.
The full source artifact retains all nine emitted native modules and every
original raw fixture output. Fresh current-PR Actions, protected merge and
released consumer acceptance remain delivery gates.
