# Known-empty Vue star exports

Issue: [#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The retained Nuxt UI application has 547 diagnostics, including 534 TS2308
collisions. Authored unaliased `export *` edges expose generated `Props`,
`Emits` and `Slots` from several Vue projections. The original application
inputs, membership, configuration and complete failure vectors remain fixed.
This bounded change does not establish complete Nuxt UI or application parity.

## Decision

Only a registered genuine Vue module with a proven empty authored star-export
set may replace an unaliased top-level `*` with `{}`. Both original inline
script blocks must be admitted by their existing OXC parse. No authored named,
type, indirect or star export may be present. A normal script exporting only a
local default component is eligible: default never belongs to the star-export
set. Authored exports in script setup, including a default or empty export
declaration, remain unproved because that form is invalid in script setup.

External script sources, unsupported languages, parser recovery, malformed
input, disabled analysis, a caller-supplied program without original parse
custody, or a lost script block keep the result unknown. Private setup types
and compiler macro declarations do not become authored public exports.

This slice changes ordinary configured batch and CLI graph finalization. The
existing editor document finalizer remains unchanged; there is no editor or LSP
acceptance claim from these laws.

The first slice admits only explicit relative `.vue` edges. The complete
projected literal must still be exactly that relative edge with `.ts` appended.
Its destination must be a currently registered virtual companion owned by the
same genuine source. Alias, package-export and extensionless routes remain
conservative, as do foreign or overwritten mirror destinations. Physical
source canonicalization and lexical mirror normalization have distinct jobs;
materializing the mirror cannot change admission.

Named exports are not expanded. Namespace exports, authored duplicates and
direct imports of generated `Props`, `Emits`, `Slots` and the typed default
remain unchanged. The TypeScript checker continues to own their diagnostics.
No generated helper export is removed and no diagnostic is suppressed.

## Parse and lifecycle ownership

The Croquis result and public option/result structs remain unchanged. An
opt-in, nonserialized observation is carried alongside the existing script
parse and statement processing. Ordinary consumers have no observation demand.
The existing SFC script draw and parser invocation counts are unchanged.

The existing import-rewrite parse records top-level bare-star spans and module
literals during its existing visitor. An opt-in statement-depth guard proves
top-level ownership without a separate statement iteration. It adds no parser,
AST traversal, filesystem graph scan, pipeline stage or serialization between
levels. Compatibility wrappers explicitly disable candidate collection.

Graph finalization replays captured spans onto the same owner's original
literal-rewrite baseline. It validates the original quote and complete mapped
literal bytes before admitting an edit. Source maps include the one-byte
expansion and clamp either generated brace to the original star. Finalization
does not parse an ordinary captured barrel again or rerun its literal policy.
Declaration and JSX finalization reuse their already existing rewrite parse.

Each source owns its admission and baseline. Re-registration replaces both;
removal drops the ownership, and a recreated source requires a new real parse.
An empty-to-named-to-empty transition therefore restores the original star
when the target becomes unknown and may erase it again after fresh admission.
Complete source/map/artifact controls cover registration order, deletion and
recreation, overwritten mirror identity and conservative routes.

## Evidence and remaining work

The original six native counterfactual packets are retained losslessly in the
separate supplemental corpus. They bind the genuine Vue and Vite provider
bodies, Panel source, generated source, configuration, actual native executable,
whole raw streams and exit status. Their original whole packet SHA256 is
`a5e32916175feeb304fc88744cb5fcd94ca5f39ee385dad5ed573022d893129e`.
They demonstrate the isolated star-export mechanism:

| Counterfactual | Before | After |
| --- | --- | --- |
| Valid consumer | Three synthetic TS2308 errors | No errors |
| Wrong prop | Three TS2308 and two TS2322 errors | The same two TS2322 errors |
| Real authored duplicate | Four TS2308 errors | The same authored TS2308 error |

Those packets edited generated source. They do not qualify this producer fix.
The additive production law discovers a separate frozen supplemental pack and
uses the ordinary configured batch checker with genuine installed providers.
It preserves complete authored vectors and separately retains raw native
generated-coordinate output over the actual materialized producer bytes.
Boundary cases prove conservative projection and retain their entire observed
native result without inventing a clean semantic golden.

At source preparation, formatting and complete corpus/hash preservation have
been checked; compilation and native execution remain pending. The existing
source-only Croquis consumption generator was rerun unchanged and its check
passed; only the Atelier SFC shard changed to reflect the actual new consumers.
That inventory is source bookkeeping and supplies no runtime evidence. Required hosted
qualification must identify the current source and emitted executable, admit
the pinned official native package and loaded provider, execute all new laws,
retain whole raw outputs and run every existing gate. An unavailable required
runtime must refuse completion. A normal shard without that runtime supplies
no native acceptance credit.

The historical 32 registrations, all existing differential fixtures and App
goldens remain byte-exact. The supplemental pack grants no historical adapter,
full fix-history, complete application, Matrix, release or performance credit.
The 10x target and original 400-workload comparison remain independent gates.
Their source enrollment must pin every changed complete producer body and
preserved oracle; an unknown, partial or drifting source delta must fail before
execution. Source enrollment supplies no native or performance credit.

TODO: qualify fresh whole application diagnostics with the original membership
and provider/configuration custody after this bounded producer repair. Prove
nonempty authored Vue export sets and alias/package routes independently before
expanding this policy. Preserve real duplicate authored exports, including
reserved names such as `Props`, before any later repair or baseline migration.
