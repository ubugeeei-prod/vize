# Selected original zero-header HTML bodies

Issue: #6838. This extends the genuine original selected-template provider
merged in #7483, without changing the diagnostic Component factory route.

## Decision

The public root entry still consumes an actual `NativeChild` from the retained
Descriptor-selected Component. It checks the original Component pointer,
direct-root parent and next ordinal before construction. Ordinary lowercase
HTML elements with no attributes may now construct their original nested
Text, Comment and similarly admitted Element children.

Only a private fused body enumerates `NativeElement::children()`. The caller
does not supply a tag, namespace, attribute vector, body, callback or nested
event list. The existing checked File factory mints the actual owner before
its body; the original iterator supplies every nested child once in authored
order. The body starts with `IncompleteChildren` and records success only
after that same iterator ends normally. Callback return alone does not prove
body completion.

A nested refusal returns through the captured private result before the
root cursor can advance. The existing sticky native state and File issue deny
the completed view even when the diagnostic canonical factory has retained
the minted owner and its actual successful prefix. Existing guarded File
walks and the root guard preserve interruption across drop, forgotten roots
and unwind. Finishing or retrying the owner cannot revive completion.

Opening and closing framing must be genuinely present. `Implicit` and
`Missing` endings refuse. Spans are derived directly from the original token
storage against the selected SourceBlock; original authored tag spelling is
retained. Namespace is internally HTML. The existing L0 stack headroom guard
runs at each recursive construction step; there is no unguarded recursive
descent, new parse, preflight tree traversal, serialization or pipeline stage.

## Bounds and evidence

Every attribute, including static attributes and directives, remains refused.
Components, SVG/MathML, uppercase names, template/slot owners, script/style,
pre/textarea/title modes, interpolation, entities, HTML Text whitespace and
If/For are unfinished. This does not legalize their diagnostic fragments or
grant Vue runtime exposure, native SFC migration or L4 output.

Seven real original-provider laws cover nested HTML page order, exact source
and owner association, explicit/self-closing/void extents, sticky nested
failure and prefix custody, foreign/reordered/nested root substitution,
unsupported boundaries, drop and forgotten/unwound completion. The existing
Element-refusal fixture now uses an actual unsupported SVG root; its entity
and unresolved-Program refusals remain unchanged.

Formatting and source gates run before publication. Fresh source-built
Actions must compile and execute these laws; earlier private or hosted
provider results do not validate this new source. Protected queue full
checks and unchanged instruction ceilings must pass before actual merge.
#6838, #6839, #6840 and all product/history gates remain open.

The reviewed storage row records the new helper's one actual arena-vector
constructor for the existing canonical Element attribute field, which is
empty in this slice. The unchanged Croquis generator updates only the L2
source shard's two additional `Span` mentions; resolved legacy usage stays
zero. Inventory classifiers, limits and numeric instruction budgets are
unchanged.
