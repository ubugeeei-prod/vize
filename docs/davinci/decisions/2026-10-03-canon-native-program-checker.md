# Opt-in native whole-Program checking

Issues: [#6849](https://github.com/ubugeeei-prod/vize/issues/6849) and
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The `vize_canon/native-program` feature enables the existing native Corsa
transport and the genuine `vize_l4` dependency. `NativeProgramChecker::check`
accepts only a borrowed sealed `ProgramProjection`. That projection retains
its original complete L2 File and sole whole-source Module unit. A caller
cannot supply another File, detached AST, document, role, language or filename.
The check result keeps that same projection borrow beside owned observations.

This bounded entry checks the original bytes emitted by L4 without another
Vize parser call, AST walk, intermediate stage or legacy virtual-TS generation.
It covers only whole Modules that the actual File provider already completes.
Unsupported typed constructs, JSX/TSX, templates and incomplete/recovered File
analysis remain provider refusals. Vue and default checker migration remain
unfinished, and the #6879 fix-history gate remains open.

The checker owns one existing CorsaBridge for an explicitly configured real
workspace with `tsconfig.json` or `jsconfig.json`. The existing compiler-option
reader and inherited-config digest are reused. A changed config chain or
preferred config identity permanently refuses that checker; changes observed
after a request retain its diagnostics with an incomplete observation. A fresh
checker is required to adopt changed settings. No scratch-config fallback is
selected by this entry.

Before opening a native document, a crate-private bridge hook initializes the
existing configured materialized-project session. It calls the existing client
activation method, whose body is unchanged; only crate visibility is widened.
Its overlay config extends the actual workspace config, retaining Module goal,
strictness and JS checking options even when that workspace initially contains
only a config file. The hook takes no caller document, URI or config argument,
is enabled only by this feature, and runs after all provider/config guards.
Default legacy calls retain their existing behavior.

The existing bridge owns URI normalization. This adapter passes its private
sealed-kind basename to `open_virtual_document` and uses the actual returned
URI for both diagnostic retrieval and closure. Failed-open cleanup uses the
same expected normalized identity; an already-prefixed URI is never used as
an input basename.

The sealed original language privately selects the emitted document extension:
JavaScript uses `.mjs` and requires actual `allowJs` and `checkJs`; TypeScript
uses `.ts`. Session-owned generated names provide no authored filename
authority. Until such a provider exists, original import/source-export rows
and every recorded Call/New/TaggedTemplate/Import invocation are refused before
backend startup. The original invocation receipt comes from genuine provider
ancestry and the existing walk, with no additional source scan.

Original comments may contain relative reference or JSDoc import directives
without producing any of those rows. The same genuine provider therefore
retains original comment-vector nonemptiness before callbacks in constant time.
Every commented Program is refused before backend startup, including ordinary
comments; comment-preserving L4 emission remains available. This conservative
boundary uses original parser metadata, without scanning or interpreting text.

Every diagnostic returned by the existing file-diagnostics transport retains
its complete original range, message, code, severity, source and related
information. Primary LSP UTF-16 ranges convert through the existing strict L0
line/UTF-16 helpers and the genuine L4 whole-source links. Invalid surrogate
boundaries, generated-only ranges, source-boundary crossings and unrecorded
links produce typed mapping refusals beside the unchanged backend payload.
Foreign related locations remain original backend observations. No unexpected
diagnostic is filtered, and cleanup failures retain successfully read vectors.

The feature-enabled backend fixture must execute the actual Corsa/TSGO bridge;
a missing required executable fails. It compares the complete projected
diagnostic vectors with actual original-source backend vectors, and checks
exact code/category/message/UTF-16 start/length/authored spelling against the
independent TypeScript 6.0.3 references. All ten original L4 reference inputs
remain registered: original commented inputs become explicit typed refusals.
Eight additional comment-free whole references cover positive and negative
JS/TS, JavaScript open-ended object assignment versus TypeScript property
errors, Unicode/CRLF, hashbangs, ordered errors and Module goal. They retain
complete independently captured diagnostic payloads. Bad-to-good session reuse
and unrecorded projections retain real backend errors or exact typed refusals.
Source/mapping laws and TypeScript captures grant no Corsa execution credit.

The first exact-source full Actions run 37108231996 reached the mandatory
feature build, then failed on a mapping-fixture import of a related-information
type that the bridge does not reexport. No backend, mapping or privacy law ran.
The test-only correction constructs the same complete related payload through
fallible deserialization into the actual field type and propagates construction
errors. Every mapping assertion and diagnostic field is retained; public and
production APIs are unchanged. A fresh corrected-source full run is required.

The corrected full run 37109531827 passed all three mapping laws and four
pre-backend refusal laws, then its actual positive Corsa law failed: the raw
virtual original `.ts` reference used inferred settings and produced global
`name` diagnostics, while the projected Module produced no diagnostics. The
empty reference workspace had no source files when the backend started, so the
existing editor fallback did not apply its authored project config. Privacy,
minimal-feature and feature Clippy steps were not reached. Both failed runs and
complete original payloads remain preserved.

The next full run 37112290503 retained mapping3 and guard4 but its actual
positive Corsa law still failed at `ts-module-goal`: byte-exact physical original
files were present before startup, yet applying the existing bridge's additional
document overlay again produced global `name` diagnostics. Physical
materialization alone did not establish equivalent Module settings. The
existing session code documents that a second overlay can detach a configured
physical document; this explains the next bounded reference correction without
claiming that the new runtime proof has already passed.

The direct disk API full run 37114821596 retained mapping3 and guard4 and
observed the actual configured project with all 18 root files, Module Force=3,
strictness, allowJs and checkJs. Its first file-diagnostic request then failed
with an explicit Unsupported response; no complete comparison, privacy or
feature Clippy law passed. An existing installed Darwin TypeScript 7.0.2 wire
observer confirmed that describeCapabilities and file/project/snapshot
diagnostic methods are all unknown. None was replaced with an empty vector.

The original reference retains the genuine configured Corsa `ProjectSession`
as its config/root-file authority and now requests closed physical-file
diagnostics through the separate existing `LspClient`. A concrete local 7.0.2
wire probe advertised diagnostics and returned the exact nonempty original
ts-property diagnostic without did_open or an overlay, then refused a query
after actual transport closure. Its first shutdown-envelope failure is retained;
the corrected probe omits shutdown/exit params, matching the SDK's existing
graceful-close method. This proves bounded transport support, not current
Linux Actions, Corsa SDK execution or complete native equivalence.

Actual backend metadata must identify the same canonical config, Module Force,
strictness, allowJs and checkJs, and each requested physical file must belong to
the returned root-file set. The existing LSP client correlates its mandatory
complete report with the exact request and properly encoded physical URI; its
initialize response must advertise diagnostics and UTF-16 positions. Missing,
null, partial or transport-failed reports refuse. Server registration and
positional configuration replies use the existing editor responder semantics;
its fixture-only worker stops before the last process owner is dropped. An
existing file absent from the open snapshot and a request after graceful actual
closure must refuse rather than synthesize success. Every primary diagnostic
keeps its original order and complete public payload.

Before any empty diagnostic case, the original fixed ts-property input must
produce its complete nonempty independent vector through both the reference
and native checker. The original ten and bounded eight families, bad-to-good
reuse and unrecorded errors remain required after that probe. Their exact
expectations are unchanged; the nonempty probe is an existing fixture, not an
alternative expected backend result.

Every frozen original source is still written byte-for-byte before backend
startup. Separate directories retain the two families' distinct sources with
overlapping names; original .mjs/.ts kinds and all independent expected vectors
remain unchanged. The native checker keeps its separate config-only workspace
and unchanged configured-session hook. No did_open, marker file, projection
marker, source rewrite or alternate language extension is used for the original
reference. All four failed campaigns and full payloads remain preserved; a
fresh corrected-source full run must execute every original/bounded comparison,
privacy law, minimal-feature check and strict Clippy before equivalence can be
reported.

The closed-LSP fixture campaign compiled and passed the three mapping laws,
but its integration target stopped at a CompactString AsRef type ambiguity
before any backend or guard law executed. The responder now uses the existing
explicit as_str accessor for that one method comparison; response bytes,
transport behavior, all original expectations and production are unchanged.
This fifth failure remains saved without backend, guard or privacy credit.
Corrected-source actual SDK and complete diagnostics remain unverified.

The existing full Rust Actions recipe explicitly runs the feature's mapping,
actual-backend and privacy tests, minimal-feature check and strict Clippy.
Exact-head Actions, current all 100 instruction probes, native Stack ancestry,
protected queue and actual merge remain required before delivery. This change
does not close either roadmap issue or any complete product/history gate.
