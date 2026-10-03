# Short borrows of original native events

Issues: #6836, #6838 and #6844.

The real lower header driver exposed a consuming-token gap: observing a
conditional operand took its non-Clone NativeAttribute, but the same live
header event was needed again by admitted_for and the lower cursor. Repeating
the original header iteration is not the consumer contract.

NativeAttribute::reborrow now returns a short immutable projection of the same
private Component, Element, Attribute and ordinal. NativeChild::reborrow does
the same for the original direct parent, SurfaceChild and ordinal. Neither
method selects a new event or stores a movable owner address. There is no public
raw constructor, Clone implementation, lookup by ordinal, parse or traversal.
The returned projection cannot outlive its borrowed original token.

The existing loop observes attribute.reborrow(), parks the normal owned operand,
then joins using the original Attribute and lets the lower cursor consume that
event once. The same pattern is the interpolation consumer prerequisite:
observe child.reborrow(), park the original expression owner, then join the
original child at the same body cursor. A short borrow does not confer lower
header/body completion or permit the cursor to consume an ordinal twice.

Four actual laws cover a single retained header token through observation,
pending growth and short stock-root join; foreign identical-source Component
refusal after reborrow; and root/nested child identity, parent and ordinal before
consuming element conversion, plus the original unquoted-slash recovery refusal.
Privacy compilations additionally check real
consumer use, short-token lifetime and the unchanged absence of Clone.

The initial new fixture build reported three ordinary API spelling mistakes;
its production pass and failed log remain retained. They were corrected against
the actual DescriptorOptions, non-Debug operand result and OpenTag token without
changing the fixture source inputs, expectations or lint scope.
The next fixture runtime preserved a genuine missing-close refusal: in
title=kept/> the slash belongs to the unquoted value. That exact counterexample
now remains a refusal law; its complete self-closing counterpart uses the actual
separating space before />. No recovered header is promoted.

This is an additive source API correction on the genuine merged conditional
provider. Fresh exact-head whole Cargo/Actions, all 100 ceilings, protected queue
and actual merge remain required. The interpolation provider and live File
header/body consumption are separate slices. It grants no native whole If/For,
target output, product migration or parser-timeout repair credit.
