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
Focused compiler controls exercise both the default and retained lowering paths.

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
