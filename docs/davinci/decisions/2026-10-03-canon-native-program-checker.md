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

The existing full Rust Actions recipe explicitly runs the feature's mapping,
actual-backend and privacy tests, minimal-feature check and strict Clippy.
Exact-head Actions, current all 100 instruction probes, native Stack ancestry,
protected queue and actual merge remain required before delivery. This change
does not close either roadmap issue or any complete product/history gate.
