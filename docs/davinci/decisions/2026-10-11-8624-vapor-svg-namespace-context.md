# Inherited SVG binding namespace (#8624)

The merged SVG repair in [#8634](https://github.com/ubugeeei-prod/vize/pull/8634)
preserves namespace when branch templates split away from their parent. Its
binding generator still used an incomplete tag list: SVG `a`, `feGaussianBlur`,
`view` and `switch` selected HTML class assignment. Adding `a` to that list would
also misclassify ordinary HTML anchors.

Carry the parser's existing namespace on `SetPropIRNode` and select SVG binding
helpers from that value. Both legacy lowering paths copy `ElementNode.ns`;
the checked native emitter currently accepts HTML elements only and retains
its explicit HTML namespace. This adds no parsing, pipeline stage or
serialization. Remove the redundant generator tag list. Review also exposed
the retained parser callback checking the child instead of its parent for SVG
integration points. Let HTML children of `foreignObject`, `desc` and `title`
use the existing Armature namespace inheritance; SVG element classification
and MathML behavior remain unchanged.

The separate `vapor-svg-namespace-8624` differential corpus contains a whole SFC
with SVG `a`, `filter`, `feGaussianBlur`, `view` and `switch`, an ordinary HTML
`a`, and an HTML `a` inside `foreignObject`. Independently authored complete
mount, update, class/style removal and unmount trees accompany its namespace,
class and computed-color vectors. The existing Actions playground browser gate executes the fresh source NAPI
producer and independently compiled stock VDOM against one pinned rc.9 browser
runtime. It compares both complete four-phase traces and every authored tree,
namespace, class and computed-color vector, SVGAnimatedString ownership, physical
bounds, style removal and retained-node disposal. The unchanged whole App and
controls are hash-checked before compilation, with separate emission digests.
The provider must be the actually loaded `.node` binding inside this checkout's
`npm/native`; retain its digest and actual Git checkout alongside both emissions.
Actions must successfully build that artifact before the existing browser test.
Focused compiler controls exercise both the default and retained lowering paths.

Exact b2286bc source Check 38151692953 successfully built the fresh NAPI artifact
but failed before any browser case: a static native import was bundled into
Vite's temporary config, redirecting relative binding resolution. Require the
original absolute entry through `createRequire`; keep the actual binding
identity, checkout and digest assertions. The same source also exposed one
canonical DOM and SSR divergence on vue-data-ui's SVG title children: the
generic L1 projection still used stateless RCDATA recording while Armature now
retained those children. The [complete failed SSR emissions and startup packet](./fixtures/vapor-svg-namespace-8624/source-failures-b228.json)
preserve both failures with exact job identities and raw-log digests. Keep all
canonical corpus comparisons active and repair the producer; these failures
grant no source or publication qualification.

The focused SVG `title` compiler control exposed a separate tokenizer defect:
the HTML RCDATA mode escaped its child markup. Resolve only an opening `title`
from each parser owner's existing live parent scope, after the name callback.
SVG parents permit children; HTML integration points (`foreignObject`, `desc`
and `title`) preserve HTML RCDATA. For those existing parser owners the query
adds no namespace stack, allocation, input walk or dependency from a level
crate to a legacy crate. Stateless raw callback consumers retain their existing
token stream. The parser owners consume a separately registered twelve-row complete ordered-tree/text
corpus, including HTML title/textarea, nested SVG, integration points,
self-closing/sibling boundaries, MathML and a similarly named ordinary tag.
Native checks additionally verify byte fidelity. These are parser controls;
they confer no published compiler or browser execution credit.

The generic L1 projection also needs this live namespace owner. Reuse the same
native component lexer with a generic namespace-recording sink. It retains
authored heads without scanning for admission or suppressing interpolation for
`v-pre`. This route uses arena-backed ancestry frames and the existing generic
structural recovery; it adds no lexer pass, pipeline stage, serialization or
legacy dependency. Version-specific header and lint policies remain in their
dialect owners. Preserve the original eleven corpus rows
byte-for-byte and append the whole SVG/path/title conditional control minimized
from the exact canonical vue-data-ui gitlink. Both native component and generic
projection consume all twelve controls with byte fidelity; Armature checks the
same complete ordered streams. Additional old/new controls preserve neutral
`v-pre`, dynamic arguments, entities, in-tag comments and exact error offsets;
the stateless facade still checks its frozen complete SVG title raw stream.
Local controls pass, but fresh full canonical DOM/SSR and protected instruction
ceilings remain required before this source is qualified.

Exact d913894 source Check 38153816463 stopped at source planning before any
compiler or browser execution: the strict storage inventory detected one new
arena `Vec` use in `parse.rs` (1/15 to 1/16). Review that site as the empty
`Vec<Infallible>` required by the shared sink interface: the neutral policy
cannot produce a boundary and skips header resolution, so this buffer never
allocates or grows. Update only its exact inventory row as required by the
storage boundary. The owner's ancestry-frame allocation is recorded above;
all storage scanners, source limits and protected instruction ceilings remain
active. This plan failure and skipped dependent jobs confer no qualification.

Exact bfda165 source Check 38154268375 passed physical Chromium and full
canonical DOM/SSR comparisons, but the unchanged L1 isolation law rejected the
generic parse route importing Vue policy. Keep that law active. Replace this
policy dependency with the generic namespace-recording sink described above;
remove its unused dialect re-export, const switch and empty admission buffer.
The strict storage ledger records the new generic ancestry owner and restores
`parse.rs` to its previous count. Preserve the whole failed tooling log identity
and passing observer job identities in the same failed-source packet. These
partial successes do not qualify the complete failed head or a publication.

The original third-party sources, six original control rows, and two authored
physical SVG controls stay unchanged. This additive namespace campaign does
not inherit their acceptance credit. Its runtime manifest reports rc.10 as
unexecuted and native coverage as zero until actual evidence exists.

TODO: after source qualification and actual merge, run this separate campaign
against a newly published included cut on Vue 3.6.0-rc.10 in physical Chromium.
Compare the entire independent VDOM trace, assert SVGAnimatedString ownership
on all five SVG binding targets and HTML className ownership on both controls,
check normalized class/style updates and removal, and retain exact package,
lockfile, emitted-module and release-source custody. The first SVG/slot release
cut proceeds independently; #8624 remains open until its actual installed
release acceptance passes.

Exact a581b18 source Check 38148305379 exposed a Happy DOM oracle defect:
its static template parser assigns SVG namespaces to HTML children inside SVG
foreignObject, desc and title. Physical Chromium and the independent VDOM
renderer assign XHTML. All whole trees, class/style/title changes and disposal
otherwise matched; the [complete failed packet](./fixtures/vapor-svg-namespace-8624/happy-dom-failed-whole-packet.json)
retains the source head/run/job and both whole traces as failed evidence.
Replace only this newly introduced emulator target with the stronger existing
physical browser gate, preserving the entire authored App, controls and all
namespace assertions. Original third-party Rust targets and their checks remain
active. No new workflow, pipeline dependency, runtime normalization or namespace
expectation exception is introduced; source and protected browser checks still
must pass, and they grant no installed/public rc.10 acceptance credit.

TODO: separately qualify retained Vapor's broader namespace callback, which
still seeds SVG-known names such as standalone HTML title from a tag list.
This packet covers inherited SVG binding targets and ordinary/foreignObject
HTML anchors; it does not confer acceptance on unrelated standalone foreign-tag
classification. Keep the current source campaign and original report denominator
fixed, and require independently authored full controls for that follow-up.
