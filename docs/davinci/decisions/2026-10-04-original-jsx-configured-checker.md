# Original JSX/TSX configured checker

Issues: #6849 and #6879. This is an explicit native API, while #6879 and the
legacy default replacement remain unfinished.

`CorsaBridge::check_original_jsx` borrows the genuine completed `JsxFile` and
its original owning `ProgramObservation` through the L4 specialized complete
projection. It preserves original JSX syntax, TS annotations, comments, imports,
exports, complete source links, and the exact `.jsx` or `.tsx` source kind. The
actual absolute existing authored path is required. A separate File or copied
source cannot supply JSX admission. No JSX lowering, synthetic global namespace,
extra parser, AST walk, virtual source write, or legacy generator is introduced.

Both original Program APIs use the same private sealed-projection checker. Its
source and root-configuration byte snapshots are checked before the worker,
after queue admission, after diagnostics and cleanup, and before publication.
The actual Corsa API resolves root membership and inherited normalized options.
The independent diagnosing LSP process attests the same absolute configuration
before and after its complete raw report. Plain JSX/TSX without original
import/export facts still requires the effective `moduleDetection: force` goal.
A wrong extension, outside path, changed source/configuration, excluded source,
incomplete projection or foreign/inferred diagnosing project remains a typed
refusal, retaining the actual owner. Existing overlay timeout and cleanup
controls are shared unchanged.

The result retains its original owner, checked URI/path/source digest,
normalized configuration, diagnosing configuration, full ordered raw report,
and one exact authored span or typed mapping refusal per primary diagnostic.
Related information is retained in the raw report. Diagnostic ranges use the
existing LSP UTF16-to-byte conversion and exact original covering source link.
The serialized L4 source map still has its existing single whole-source-start
anchor; complete token/line-anchor source maps are unfinished.

The independent primary TypeScript 6.0.3 probe uses the actual pinned Vue 3.5.35
`import 'vue/jsx'` declaration and an ordinary existing configured project. Both
JSX and annotated TSX report TS2322 for `<div id={1}>`, including its actual Vue
`runtime-dom.d.ts` related declaration, followed by TS2339 for `value.missing`.
JSX preserves literal type `1`, while the genuine TSX `: number` annotation
retains type `number`. There are no fabricated JSX declarations or filtered
vectors. This probe is oracle evidence; genuine source-built Canon/TS7 Actions
must separately pass the full raw vector and original owner laws.

New hosted laws check both true source kinds, non-BMP comment and CRLF UTF16
ranges, original Program body identity, the entire actual raw TS7 vector, exact
source bytes, correct authored mappings, source digest and same-project receipt.
They also check wrong kinds, relative/outside paths, stale source, actual
configured membership and Module goal, and an externally revised source while
the real diagnosing process starts. The latter must refuse publication and reap
its owned process. Existing original JS/TS and native Vue laws remain intact.

The source dependency is the genuine L4 JSX/TSX projection PR #7624. The consumer
branches from that real provider head and must register a native Stack before
protected admission. Source-head Actions, protected full workspace suites,
actual 100 × 3 instruction measurements and immutable ratchets, and literal
signed merges remain necessary before delivery credit. No manual full/100 runs
or local Cargo builds are needed.

Coherent snapshots across an entire imported/configuration graph, broader
incomplete lower JSX/TSX families, editor integration, fix-history equivalence,
complete serialized source maps, and default replacement remain unfinished.

Independent source review corrected two fixture assumptions before validation:
the actual pinned Vue declaration indents `id` with four spaces, and Oxc
`SourceType::tsx()` requests Unambiguous until `with_module(true)` establishes
the original Module profile. The laws use the real declaration offset and
explicit original TSX Module; production eligibility is unchanged.

The corrected provider source `2cbffbd790` passed exact Check37140046811,
all four Rust runtime workers and every required source context. Native
Stack #7628 reports size two, positions #7624 then #7627, with both source
OIDs unchanged by complete-URL registration. Only the green bottom was admitted
through `gh stack merge 7624 --yes --squash`, after read-only clean unions with
both older queued sources. This is queue admission, not an actual merge.
Independent bounded consumer review found no additional production blocker
after the genuine fixture corrections; full-vector TS7 Actions is still required.

The first genuine source-built Canon campaign (e3b758d37d, Check37140842717)
compiled the actual consumer and owner-drop doctests. The actual unfiltered
TS7 JSX report contains exactly TS2322 and TS2339 with the independently
verified primary codes, messages, and UTF16 ranges, but **no relatedInformation**.
The TypeScript 6.0.3 primary oracle includes the actual Vue declaration. The
existing shared LSP initializer does not advertise related-information support;
this campaign establishes the output difference, not its sole cause. The new
law now asserts the complete actual raw TS7 vector without adding or filtering
fields, and the related-declaration equivalence gap remains unfinished under
#6879. No shared initializer or legacy behavior was changed.

The identity law also created a wrong-kind file with the same basename as its
true JSX/TSX file. It now uses and removes a distinct `wrong.*` fixture before
configured membership/Module checks, and prints the actual typed refusal on a
failure. The first externally revised JSX/reaping law passed separately. Fresh
source Actions is required after these changes; the consumer was never queued.
