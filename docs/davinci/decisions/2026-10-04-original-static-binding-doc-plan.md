# Original static binding expression and whole-SFC Doc plan

This is a private source-grounded design, with no implementation, build,
publication or queue admission. Its inspection snapshot is
0fe5c0753ca9de2797e128a525a2a7c4eb2a34f2. The original attribute-head provider
is genuinely accepted in [#7768](https://github.com/ubugeeei-prod/vize/pull/7768),
signed 9702c014e06a268d2651202b7fcb951db7e729df. The immediate whole-SFC
writer extension depends on the conditional consumer in
[#7774](https://github.com/ubugeeei-prod/vize/pull/7774), which is source-green,
review-ready and unqueued at this snapshot, not accepted. Its full source
proof does not transfer to this proposal or to protected validation.

## Existing authentic seams

NativeAttributeHead stores the actual selected component, current original
attribute, checked authored name block, one dialect DirectiveName and its
conditional selection. Its constructor checks foreign/recovered/verbatim
precedence before decomposition. The original NativeAttributeExpression
owns an Origin, NativeConditionKind and RetainedExpression. Its kind is
If/ElseIf; it cannot be renamed or relabelled as a binding receipt.

The shared observe_conditional tail already derives Origin from the original
attribute, calls prepare_attribute_value once, then parse_once with Shape::Expr
and the selected Module JS/TS profile. Syntax holes remain whole retained
observations. NativeAttributeValue is a separate decoded value receipt with
complete token frame and no expression authority. Calling that generic value
observer and an expression observer for one binding would decode twice.

Native events confirms it has no active L1 binding/head/value/Origin edits.
Its pending L2 File value receiver still refuses directives. Neither this
provider nor formatting a binding value legalizes that File or a target.
Vue 2's private whole-SFC layout owner is separate; this proposal does not
edit Expr Context/Origin, Vue 2 text/projector or Vue 2 template layout.

## Lowest provider: a sealed static binding head and distinct owner

Add a short, non-Clone NativeStaticBindingHead borrowed only from an actual
NativeAttributeHead. The old head's stored fields and all conditional APIs
remain unchanged. NativeAttributeHead::static_binding returns:

- Ok(None) for another directive family or a plain name;
- Ok(Some(sealed view)) only for a complete nonempty static argument spelled
  :arg or v-bind:arg, without modifiers;
- Err(UnsupportedDirective) for a recognized bind family whose shape is not
  admitted, including Prop shorthand, dynamic/no argument and modifier runs.

The dialect helper consumes the head's existing DirectiveName metadata,
never another decompose call or a name scan. It checks prefix Bind, or Full
with the original typed name span equal to bind. ArgSyntax::Static alone is
insufficient: the current dialect can overwrite a closed dynamic argument
with a trailing static segment. Require the argument start to equal name
block start + 1 for Bind or typed directive name end + 1 for Full; require
argument end equal name block end, nonempty argument and empty modifiers.
Use checked addition. Thus :[key]tail and v-bind:[key]tail are refused without
reinterpreting their trailing bytes as an ordinary static argument.

The sealed view retains the original head borrow and checked argument span.
Its readonly getters expose those original facts. Its consuming
observe_expression method prepares/parses the current value once and returns
NativeAttributeBindingExpression. No caller-selected AST, language, map,
source slice, argument span or ConditionKind can assemble the owner.

The distinct movable owner stores private Origin, the original argument span
and one RetainedExpression. It exposes raw/name/value/argument spans, syntax,
and a short admitted_for join with the actual selected component and current
attribute. Use the unchanged Origin::matches checks, including original
component/element/attribute identities, ordinal, template index, grammar,
full block/source identities and exact raw name/value windows. Syntax
admission is then borrowed from that same retained stock owner. Moving out
syntax explicitly discards binding/header authority.

Factor only a private original expression-value preparation tail returning
the existing Origin and RetainedExpression. Both the old conditional wrapper
and the new sealed binding observer use it once. Retain the exact old
conditional kind, owner fields, failure type, public API and refusal order.
Reuse NativeAttributeExpressionFailure for actual preparation/parser failure:
it has no ConditionKind field and already normally owns any unexpectedly
shaped NativeSyntax. Do not manufacture a surrogate Expr after a failure.
Unsupported head selection has no prepared expression or syntax owner;
the actual selected component retains its whole original header/source.

Quoted values are the first formatter product boundary. The L1 owner does
not confer quote or target legality: a complete unquoted value can be
observed honestly, then refused by the formatter with that owner retained.
A valid static head without a value returns actual IncompleteValue before
preparation. Shorthand source is never expanded into a fabricated value.

## Immediate consumer: the existing whole scriptless SFC writer

Add only an explicit FormatConditionalsAndStaticBindings variant to
NativeSfcDirectivePolicy and NativeTemplateValuePolicy. Existing fields and
Refuse/FormatConditionals/PreserveOpaque/RefuseValuedDirectives contracts,
including native default strict refusal, remain unchanged. All captured
descriptor/print/directive options still contribute to the whole result.

Use the same existing original child traversal and eligible opening-header
route. Each actual NativeElement.attributes item obtains one original Head
and its checked name Doc. Only the new policy selects the sealed static
binding view; old policies do not prepare or admit bindings. Retain the
old recovery/verbatim negative route and source/name/value-token precedence.
No preobserved list, second attribute/body pass, root move, decode, parse,
AST normalization or legacy formatter route is added.

For a complete value, use the existing attribute_with_name value callback
after its existing name/eq/open/content token and source checks. Hold the
selected view in Option and take it once inside the existing FnMut callback;
no clone or callback signature change is needed. Consume the view once,
park the actual success or failure in normal owning storage, then check
quote framing, actual original admission and expression_document. An
unquoted admitted binding is retained before UnquotedBindingValue refusal.
Defer an unsupported static-selection error until that checked callback;
then return BindingHead at the original authored name span without preparing
an expression. Earlier original head/name/layout failures still win. The
existing closing-quote check remains after the callback as it is today.

The callback is not invoked when a value is absent. Therefore the new policy
must handle a valid static binding head with no value after that same
attribute_with_name event completes the original name/eq token checks:
consume its sealed view, retain actual IncompleteValue failure,
and stop before later attributes/children. This is distinct from unsupported
dynamic/noarg/modifier head selection, whose deferred error is also returned
after those checks. Existing conditional missing-value
behavior and old-policy refusal order are not silently changed.

Keep the current conditional attributes Vec and attribute_failure fields
unchanged. Add separate normal owning binding Vec and binding_failure,
alongside the existing interpolation Vec/failure. Each family's index remains
its own ordered observation ordinal; current conditional indices are not
all-header ordinals and must not be renumbered by intervening bindings.
Add separate typed BindingHead/BindingObservation/BindingRejected/
BindingExpression/UnquotedBindingValue/MissingBindingOperand refusals rather
than disguising them as conditional attributes. Stop at the first original
failure; never replace it by a later family failure or a successful prefix.

On both lower outcomes, internal into_observations transfers all three
families and their actual current failures to NativeSfcObservation before
root framing. The existing Descriptor/selected/options/outcome fields and
all current public interpolation/conditional accessors remain intact; add
binding accessors. The complete Doc remains outer prefix + template Doc +
outer suffix under one lookahead and one print, with full code/changed/options
and typed refusal. No formatted template-only result is spliced into source.

Existing document into_full_parts (four tuple) and failure full parts (six
tuple alias) cannot transfer added binding storage without changing their
contracts. Preserve those exact schemas and explicitly document that old
transfers release new binding custody. Add new complete binding-aware named
parts/transfer APIs containing selection, all three normal vectors, Doc or
refusal, and each actual failure. Never put Drop-bearing observations into
the no-Drop arena carrier. Keep compact failure payloads normally Box-owned
where needed; None allocates nothing and no lint suppression is introduced.

For the internal header helper, pass the existing Builder and current
NativeElement/parts/depth instead of adding an eighth free argument to
conditional::open_element. Its existing selected/input/cursor/allocator/value
policy fields are reborrowed at that same event. This avoids copying policy
into observation proof or creating another traversal. Add a small binding
value helper; keep the conditional visitor and its current outputs intact.

## Reviewable source layout

Provider, independently reviewable on fresh actual main:

- new L1 markup/native/operand/binding.rs: distinct normal owner and join;
- new L1 markup/native/operand/binding/head.rs: sealed short head transport;
- new dialect/vue3/operand/binding.rs: typed static-argument selection;
- tiny operand.rs/head.rs/dialect operand module and markup reexports;
- private shared original expression-value tail in operand.rs;
- new binding tests for observations/refusals/geometry/ownership and CF laws.

Consumer, a real dependent child from the provider head, after #7774 is
actually accepted or with its genuine dependency separately registered:

- new Glyph selected_template/binding.rs value-to-Doc helper;
- existing Builder/conditional helper/Input/Observed success and failure
  modules: same event plus additional normal binding storage/transfer;
- existing selected-template and SFC policy/refusal/observation/build modules
  and tiny native_doc exports; no shared Expr/Context/Origin edits;
- new native_static_binding_sfc integration root with separate layout,
  preservation, policy, refusal and custody helpers, each <= 350 lines.

The provider needs actual #7768 APIs, not the unaccepted Glyph implementation
in this inspection snapshot. It must start from fresh literal main and carry
only its own delta. Publish the consumer with the true provider as PR base,
register the native Stack and verify ordered membership. Do not mutate a
healthy individually queued parent to simplify publication. After an actual
prefix merge, replay only the remaining child onto fresh actual main and
qualify its exact head again. The present #7774 stays frozen and unqueued
while fixed v0.432.0 publication is coordinated.

## Independently authored whole vectors and controls

These expected bytes come from the existing Doc rules and original input
spellings, not formatter capture. They are proposals, not executed fixtures.
Use native combined policy, width 200, indent 2 and generated LF unless noted.

```text
input:    <template><p :id='a+b'>{{1n}}</p></template>
expected: <template><p :id='a + b'>{{ 1n }}</p></template>

input:    <template><p v-bind:id="a+b">{{2n}}</p></template>
expected: <template><p v-bind:id="a + b">{{ 2n }}</p></template>

input:    <template><p v-if='a+b' :id='c+d'>{{1n}}</p><p v-else-if='e+f' v-bind:id='g+h'>{{2n}}</p></template>
expected: <template><p v-if='a + b' :id='c + d'>{{ 1n }}</p><p v-else-if='e + f' v-bind:id='g + h'>{{ 2n }}</p></template>

input:    <!--雪--><template><p :id='a&#43;b'>{{1n}}</p></template><!--尾-->
expected: <!--雪--><template><p :id='a &#43; b'>{{ 1n }}</p></template><!--尾-->
```

Additional complete source-derived vectors use literal JSON string escapes:

```text
input1: "<!--前-->\n<template><p :id='a+b' v-if=\"ok\" v-bind:title='x&#43;1'>{{n*2}}</p></template><!--尾-->"
expected1: "<!--前-->\n<template><p :id='a + b' v-if=\"ok\" v-bind:title='x &#43; 1'>{{ n * 2 }}</p></template><!--尾-->"
input2: "<!--é--><template><p v-bind:雪='n&#43;1' :id=\"x+y\"/></template><!--z-->"
expected2: "<!--é--><template><p v-bind:雪='n &#43; 1' :id=\"x + y\" /></template><!--z-->"
```

The first retains binding/conditional/interpolation counts 2/1/1, with
conditional ordinal 0 despite the earlier binding. In the second, original
name/argument/value spans are 22..32 / 29..32 / 34..41. Its complete map is
Identity 0..1 -> 34..35, Entity 1..2 -> 35..40, Identity 2..3 -> 40..41;
decoded value n+1 and original argument 雪 remain separately observable.

Full flat self-closing width is 49 scalars for the next vector. Width 49
uses the first expected source; width 48 uses the second. A broken header
line needs 13 scalars including indent and its closing quote; width 12 uses
the third. Verify widths 49/48 and 13/12 plus zero, not a substring outcome.

```text
input:    <template><p :id='a+b'/></template><!--tail-->
expected49: <template><p :id='a + b' /></template><!--tail-->
expected48:
<template><p
  :id='a + b'
/></template><!--tail-->
expected12:
<template><p
  :id='a +
    b'
/></template><!--tail-->
```

Author complete physical LF/CRLF comment vectors using original
`/*雪*/` + physical newline + `(α&amp;&amp;β)//尾` + physical newline.
Preserve those physical bytes while independently varying generated LF/CRLF,
indent 0/2/4 and width. Assert complete raw and decoded values, full map
segments, typed comments, original Expr spans/root and whole output. Derive
file-absolute name/argument/value coordinates from complete original UTF-8
input; keep selected-local layout offsets and decoded Expr spans distinct.

Two complete cross-ending vectors make physical custody distinct from
generated layout. These strings use literal JSON escapes, width 200 and
indent 2; author all four physical/generated combinations before execution.

```json
[
  {
    "physical": "LF",
    "generated": "CRLF",
    "input": "<template><p :id='/*雪*/\n(α&amp;&amp;β)//尾\n'>{{1n}}</p></template>",
    "expected": "<template><p\r\n  :id='/*雪*/\n(α &amp;&amp; β)//尾\n'\r\n>{{ 1n }}</p></template>"
  },
  {
    "physical": "CRLF",
    "generated": "LF",
    "input": "<template><p :id='/*雪*/\r\n(α&amp;&amp;β)//尾\r\n'>{{1n}}</p></template>",
    "expected": "<template><p\n  :id='/*雪*/\r\n(α &amp;&amp; β)//尾\r\n'\n>{{ 1n }}</p></template>"
  }
]
```

The following complete negative vectors freeze intended first-refusal
metadata independently from their UTF-8 spellings. Ranges are file-absolute
authored byte spans; layout offsets are selected-template local. These are
design expectations, not observed execution results.

| Whole input                                        | First whole-product refusal                               | Current binding custody                 |
| -------------------------------------------------- | --------------------------------------------------------- | --------------------------------------- |
| `<template><p :[key]='a+b'/></template>`           | BindingHead, name 13..19, UnsupportedDirective            | No prepared expression                  |
| `<template><p v-bind='a+b'/></template>`           | BindingHead, name 13..19, UnsupportedDirective            | No prepared expression                  |
| `<template><p :id.prop='a+b'/></template>`         | BindingHead, name 13..21, UnsupportedDirective            | No prepared expression                  |
| `<template><p .id='a+b'/></template>`              | BindingHead, name 13..16, UnsupportedDirective            | No prepared expression                  |
| `<template><p :id/></template>`                    | BindingObservation, name 13..16, index 0, IncompleteValue | Whole actual failure, syntax None       |
| `<template><p v-bind:id/></template>`              | BindingObservation, name 13..22, index 0, IncompleteValue | Whole actual failure, syntax None       |
| `<template><p :id=a+b></p></template>`             | UnquotedBindingValue, value 17..20                        | One real binding expression, no failure |
| `<template><p :[key]tail='a+b'/></template>`       | Template SourceMismatch, offset 3                         | No prepared expression                  |
| `<template><p v-bind:[key]tail='a+b'/></template>` | Template SourceMismatch, offset 3                         | No prepared expression                  |
| `<template><p :='a+b'/></template>`                | Template Unsupported Directive, offset 3                  | No prepared expression                  |

The two dynamic-plus-tail spellings are independent provider selection
negatives: static_binding must return UnsupportedDirective on their original
metadata. The whole writer already checks name_document before the value
callback, so its original head/argument geometry refuses first as
SourceMismatch. The empty argument similarly fails the existing name Doc.
Preserve both controls and that earliest-owner distinction, rather than
moving binding selection ahead of the original layout checks. Selected
component custody remains complete in every row.

Keep separate full-source negative controls for :[key], :[key]tail,
v-bind:[key]tail, v-bind with no argument, :id.prop, .id, empty argument,
:id and v-bind:id without value, admitted unquoted :id=a+b, unsupported
Spread/TS/call descendants, L0/L1 admission limits, Glyph depth refusal,
foreign/sibling/equal-source receipts, recovery/v-pre, scripts/styles/other
dialects and unknown outer roles. Preserve default strict and existing
FormatConditionals refusal on a quoted binding and bare-template opaque output.

Whole failed sources must include a successful earlier binding, conditional
and interpolation, then each later failure and unvisited trailing children.
The [private full-source vector record](../plan/original-static-binding-doc-vectors.json)
freezes seven such sources and exact original prefix/current custody metadata.
Assert every complete retained prefix/current failure and original remaining
component source; no partial source assertions or fabricated syntax owner.
Test moves, Vec growth, named complete transfers, dropping short views,
normal drop/caught unwind and deliberate forget boundaries. CF laws reject
forged owner/view, Clone/Copy and escaping original/owner borrows. No
destructor-count, JS evaluation or native completion credit is invented.

Every positive checks complete code/changed/captured options, repeat format
on the same owner, an independently parsed fixed point in another arena,
typed semantic fingerprint and exact authored comments. Expected source and
first failure metadata are authored before hosted execution.

## Delivery gates and paired records

Freeze provider and immediate consumer source/laws for root and independent
retained-source review before publication. Carry the decision on both #6836
and #6847 and in the owned central formatter paragraph in each genuine
source change; keep detailed history in linked companions, central <= 350
physical lines, preserving unrelated records and current actual issue state.
Do not publish a separate record-only progress PR during the release hold.

Regenerate only actually changed canonical consumer/source inventories from
the current source. Preserve all existing conditional/head/value/Event/For,
Doc, interpolation, 21 conditional-SFC and 29 strict-SFC controls. Use fresh
exact-head Actions for meaningful provider/consumer laws, CF/API/Clippy,
immutable formatter history/API/CLI packets and protected full suites.

Preserve existing L0 nesting 31 and numeric 4096-byte guards, L1 31-unit admission, Glyph
expression Doc depth 16 and template depth 128. Protected instruction gates
keep all original 100 ratchets and four formatter ceilings; no increased
budget, calibration, oracle changes or repeated manual full campaign.
Those existing legacy formatter workloads do not benchmark this new native
binding route. Track every protected candidate and literal signed merge;
remove only an exact known-red own candidate until a reviewed fix is ready.
Scripts/styles/dialects, L2 File/semantic binding lowering, shared native
equivalence/default replacement and legacy deletion remain unfinished.
