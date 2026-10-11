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
class and computed-color vectors. The existing locked rc.9 VDOM oracle compares
the product's complete runtime trace and every authored tree and namespace phase.
The class and computed-color vectors are separate physical rc.10 assertions;
the source runtime does not claim browser class ownership or computed styles.
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
