# Original static binding expression provider

This genuine L1 source slice starts from literal main
caee1656927e1232f4aaabff2328246decb8bc58. It implements the provider half of
the [reviewed binding plan](./2026-10-04-original-static-binding-doc-plan.md)
for #6836 and the immediate whole-SFC consumer in #6847. At this private
freeze, source execution, publication and protected acceptance are pending.

## One original selection and distinct owner

NativeAttributeHead::static_binding borrows the existing actual Head. Its
sealed non-Clone NativeStaticBindingHead retains that short borrow and the
original argument span. It reuses the existing dialect DirectiveName, checks
the exact prefix/argument/end/modifier geometry, and never decomposes or scans
the attribute name again. Plain names and other directive families return
None; dynamic/noarg/modifier/Prop shapes return UnsupportedDirective.

The exact argument-start guard also refuses :[key]tail and
v-bind:[key]tail: their existing dialect metadata can contain a trailing
Static segment after the dynamic argument. That segment cannot be substituted
for the complete original binding argument. Empty arguments refuse.

Consuming the sealed view prepares the actual value once and performs the
existing stock expression parse once under the selected Module JS/TS grammar.
The shared private expression-value tail contains the original conditional
origin/preparation/parse/error-mapping body byte for byte. The conditional
wrapper reconstructs the same Origin, NativeConditionKind and syntax fields;
its public API, original fields and refusal priority are unchanged.

NativeAttributeBindingExpression is a distinct normally owned receipt with
the same private original Origin, checked argument span and RetainedExpression.
It never relabels a conditional kind and never calls the separate generic
decoded value observer. Source, decode map, diagnostics, comments and local
syntax holes stay in the whole stock observation. An absent or incomplete
value returns the actual shared failure, retaining any real parser artifact;
it never fabricates a value or neutral Expr for shorthand.

Its short admitted_for view joins only the original component/header/ordinal,
template index, profile and physical source windows through unchanged
Origin::matches. Foreign/sibling/independent same-buffer/equal-byte copied
headers cannot confer that authority. Moving out syntax deliberately releases
binding/header authority. The selected owner may end after the short view;
the retained stock expression remains in normal owning storage.

## Product boundaries and required proof

L1 retains complete unquoted values honestly. Quote legality is the future
formatter consumer's responsibility, after the original value token checks
and normal owner custody. Provider selection and whole-writer earlier name
Doc failures remain separate: dynamic-plus-tail heads are provider negatives,
while the current checked name Doc can refuse their framing first.

Seven compile-fail examples preserve non-Clone/field/borrow boundaries for
the short view, movable owner and admitted join. Dedicated provider laws
cover complete original observations, geometry and physical comments,
intrinsic grammar profiles, source/shape failures and original lifetime joins.
These authored laws and all existing conditional/head/Event/For/value controls
require fresh exact-head hosted execution; private source review is not runtime
proof. No local Cargo build, runtime execution, install or campaign is used.

The immediate consumer is a separate true provider child and must retain
genuine #7774 conditional implementation ancestry. It observes during the
same existing original attribute/child visit, parks binding observations and
actual failures in a separate normal vector before quote/admission/Doc
failures, and extends one complete scriptless SFC Doc under an explicit policy.
The [complete design vector record](../plan/original-static-binding-doc-vectors.json)
keeps authored prefix/current-failure expectations separate from execution.

Do not copy conditional helpers or admit an individually queued dependency
by prose alone. Register and verify the real native Stack, then qualify each
exact source/prefix and protected candidate through literal signed merge.
Existing #7774 delivery remains separate from this private implementation.

All L0/L1/Doc guards, immutable formatter history, original 100 ratchets and
four formatter ceilings remain unchanged. No new stage, serialized bridge,
legacy-backed route, oracle adjustment or default switch is added. Scripts,
styles, other dialects, L2 File/semantic binding legalization, native shared
equivalence/default replacement and legacy deletion remain unfinished.
