# Original selected-template DOM emission

Paired issues: [#6839](https://github.com/ubugeeei-prod/vize/issues/6839) and
[#6840](https://github.com/ubugeeei-prod/vize/issues/6840). This consumes the
actual selected-root ownership, ordinary body and original static-header
providers; it does not replace a product's legacy path or close its history
gate.

## Same owner, existing encoder

`emit_template` accepts only `&NativeTemplateDomAnalysis`. The wrapper keeps
its moved `NativeTemplateView` and private `NativeFileAnalysis`, including
the original selected Component, complete File and canonical artifact.
Every encoder input comes from that same wrapper. The existing sole L3
decision walk and private L4 `encode` are reused; no neutral analysis is
extracted or rebuilt, and no source parse, AST visit or stage is added.

The literal-only File expression writer now keeps a private borrowed File
and its already-built DOM facts. Only the two typed entrypoints construct
this projection. Exact File/node/scope/AST/source/span/coordinate checks,
literal-only shape and empty reference tables stay unchanged. The selected
entry grants no caller policy or Vue runtime read exposure. Existing File
refusal laws retain their original inputs and goldens.

Static properties preserve canonical authored order. Bare and explicitly
empty values both emit `""`; other values keep original Unicode, quotes,
backslashes and newlines through the existing quoted writer. Anonymous
property key/value links point to the complete authored Attribute span.
Attributes create no node ids, expression rows, dynamic flags, normalization
helpers or semantic side tables.

The pinned original Unicode-name fixture exposed an existing byte mismatch:
the ASCII-only key writer quoted `名`, while the reference emits that valid
IdentifierName directly. Property spelling now uses the existing workspace
stock syntax predicate, including Unicode; invalid identifiers remain quoted.
The original input, complete pinned code/map and full-span link expectations
stay authoritative. This is lexical encoding, with no expression parse or
binding/tag-role admission, and needs fresh unchanged instruction gates.

## Structural HTML and target eligibility

The neutral L2 provider remains unchanged: an original HTML `search`
Element can complete structurally. Pinned Vue 3.5.35 resolves that spelling
as a component, so the same existing L3 DOM Element event records the typed
`ElementRole` refusal at its actual node/span. The original Op, completed
File, source and Frame observations remain available. L4 consumes that
fact and returns no Writer, for either link sink.

The dev oracle audits the actual L0 HTML table against the pinned compiler,
excluding the six already-refused special modes. `search` must remain the
only component-role mismatch for this bounded family. This negative target
refusal is not a full Vue tag-role provider and cannot be promoted into lint,
SSR or type-check admission. Those targets retain their own genuine facts.

## Genuine execution laws

Six original descriptor-to-selected-owner pipelines cover ordered nested
headers, UTF-8 values/names, quote/backslash/newline escaping, bare versus
empty values, original Text/Comment roots, an empty template and nested
siblings. They pin independent node/attribute counts and all sixteen
complete UTF-8 Attribute extents, original token/Component/Element/ordinal
identity, canonical DomNode Op identity and dense node ids.

Recorded and NoLinks emission must produce identical complete module bytes
and helper order. Every static property key and value retains its exact
anonymous authored link. Maps retain the whole original SFC source. Late
root/nested header refusal and root drop/forget/unwind deny a completion
view, retaining the actual prefix and original header. Separate complete
`search` root/nested laws retain structural ownership but refuse target
emission at the original location.

The guaranteed first merge-group tooling shard runs a named CI-profile
Rust capture law after the existing CLI/runtime preparation. A reusable
action executes the resulting native modules against the exact pinned
compiler/runtime. Missing, empty, incomplete, reordered or foreign captures
fail. The Node proof imports captured native bytes, checks independently
regenerated complete reference bytes/raw maps and runtime trees, and uses
throwing contexts to reject hidden reads. It retains the actual native
module/map capture as an artifact. No new job or pipeline stage is added.

Formatting, ordinary module discovery, storage and generated source-count
checks precede publication. Fresh exact-head source Actions, captured queue
execution, full merge-group suites and unchanged instruction ceilings must
pass before actual merge. Historical fixture execution gives this entry no
execution credit.

This emits bounded static template render declarations. Helper import/export
assembly does not grant script emission, setup access, whole-SFC output or
default product migration. Directives, special attributes, controls,
entities/whitespace, namespaces/components, other Vue dialects and full
upstream source-map equivalence remain unfinished. Roadmap and product-history
issues remain open.
