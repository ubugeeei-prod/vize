# Original import facts in complete Canon documents

The explicit `original-import-products` feature lets a caller retain the
original script Program and reuse its L2 import facts in the complete existing
Canon virtual TypeScript generator. `ScriptDeclarations` now exposes original
import-statement selections, including side-effect imports, and shares its
existing local-name classification with the statement view.

`OriginalImportFacts` requires the exact TS unambiguous profile used by the current
import-slice readers and a complete original L1 parse. It preserves original
statement ranges, raw local spellings (including escapes), and syntactic
type-only status. Generation validates the exact original script buffer and
every import statement selected by the existing summary. Resolved value/type
identity remains a separate project query; these syntax facts do not admit a
native File.

`generate_virtual_ts_with_original_imports` consumes those facts for value
imports, syntactic type-only imports, and component-name shadowing. It keeps the
existing binding filter, script value-use filter, generation, and mapping model.
Type-only sets keep their existing demand gates, and scripts without value
imports keep their allocation-free binding lookup. The resource screen covers
props demand, unused mixed imports, and unused type-only imports separately.
The default generator and its recovery/retry paths retain their current readers.
The new input is scoped to its actual script buffer: a synthetic merged script
cannot claim the coordinates or ownership of an authored SFC block.

The dedicated Actions workflow compares full generated text and the complete
mapping packet, pins elimination of import-slice parses on an original fixture,
and retains the exact revision, original source manifest, host/compiler, coverage
and refusal counts, and raw paired resource results. Original-source qualification
selects one physical script from each admitted descriptor and explicitly reports
other selections and syntax refusals. Prepared measurements exclude original
Program and import-fact construction. Cold measurements include the same original
Program observation on both sides and import-fact construction on the candidate.
Both use the existing 31-pair/A/A protocol and exact allocation/peak-byte gates.

This scoped adoption does not delete legacy allowances, select a production
backend, qualify merged scripts, or replace full real-project and default
qualification or its actual post-implementation zero-fallback observation period.
