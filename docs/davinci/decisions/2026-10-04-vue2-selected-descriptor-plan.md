# Original Vue 2 selected descriptor: bounded ownership plan

Private design for [#6837](https://github.com/ubugeeei-prod/vize/issues/6837)
and [#6842](https://github.com/ubugeeei-prod/vize/issues/6842), frozen against
literal main `6987c523ecd2bee6e8489e2973e4b7499ce25018`. The paired issue
text is [prepared locally](./2026-10-04-vue2-selected-descriptor-issue-draft.md)
for root review; it has not been posted. No production implementation,
build, publication, campaign or queue admission accompanies this plan.

## Current original providers and gap

`container::vue::descriptor::Policy::new` currently refuses every version
except `VueVersion::V3`, every dialect except `VueDialect::Vue`, and the
experimental in-tag-comment option. Its original `split_with` callback
records each actual block index, checked root/content `SourceBlock` and
matched closing-name span once. The public capture cannot create an
`AdmittedDescriptor`. These modern routes and refusals must remain intact.

The merged `dialect::vue2::surface::ComponentParse` is a distinct owner.
`parse_component_block(allocator, block)` already runs the original native
Component lexer and Vue 2 sink once over that genuine block. It retains
the whole source frame, CST, markup errors and normally owned original
text/filter/base/argument syntax. `children` and `text_for` retain actual
Component/parent/ordinal membership, recovery, token geometry and native
holes. The borrowed expression provider is also present on this base.
Those providers currently have no historical selected-descriptor owner.

This design joins those actual providers. It never constructs a modern
`NativeTemplateComponent`, `NativeTemplateOwner`, `NativeTemplateFile`,
`AdmittedDescriptor` or a caller-paired Component/source tuple.

## First explicit family

The proposed entry accepts `DescriptorOptions` but admits only
`version: V2`, `dialect: Vue` and `SurfaceParseOptions::default()`.
The existing Component entry intrinsically selects the default `{{`/`}}`
delimiter and JS expression/filter profile; no caller grammar is substituted.
V2_7, V1, V0_11, V0_10, V3 and PetiteVue refuse in this new entry.
Accepting scriptless V2_7 would be a later explicit profile decision.

Require exactly one original normal lowercase `template` block. Reuse the
existing template attribute policy: no attributes or one genuine `lang=html`
is supported; duplicate/encoded/wrong-language/ambiguous/unknown attributes,
`src`, valued or unsupported role attributes refuse. Refuse every script
(ordinary/setup, empty/comment-only, JS/TS or external), every style and
every custom block at its actual splitter event. Preserve each original
block and splitter diagnostic. Missing, duplicate, self-closed, uncertain,
unclosed and noncanonical opening spellings refuse. Closing-name case and
whitespace remain the original matched span, as with the current splitter.

The splitter's existing treatment of outside-block text/comments remains
unchanged and inspectable in the full source. This is not a claim of whole
upstream Vue 2 descriptor equivalence, arbitrary document grammar or custom
delimiter support. Body grammar and TextView admission remain the existing
Vue 2 provider's separate checks.

## Sealed ownership and proposed API

```rust
Vue::observe_vue2_descriptor<'a>(
    &'_ self,
    allocator: &'a Allocator,
    original_source: &'a str,
    options: DescriptorOptions,
) -> Vue2DescriptorObservation<'a>;

impl<'a> Vue2DescriptorObservation<'a> {
    fn selected(&self)
        -> Result<Vue2TemplateView<'_, 'a>, Vue2DescriptorRefusal<'_>>;
    fn component(&self) -> Option<&vue2::surface::ComponentParse<'a>>;
    // Immutable original source/options/root/container/issues/errors metadata.
}

impl<'o, 'a> Vue2TemplateView<'o, 'a> {
    fn component(&self) -> &'o vue2::surface::ComponentParse<'a>;
    fn block(&self) -> SourceBlock<'a>;
    fn container_index(&self) -> usize;
    fn opening_name(&self) -> Span;
    fn closing_name(&self) -> Span;
}
```

Names above are proposed APIs, not available methods. The observation has
private fields owning the one original `Container`, checked `SourceRoot`,
options, original policy issues, selected `TemplateSelection` and optional
normally owned `ComponentParse`. The view borrows that whole observation;
it is non-Clone and has no public factory, extraction into a modern view,
mutation or caller-supplied Component/Block/Span pairing.

`selected()` certifies the supported original envelope and the actual
once-created Component association. It does not certify clean body grammar:
the stored Component's original errors, unsupported boundaries and native
holes remain available, and `text_for` applies its unchanged admission.
Invalid envelopes return the whole original observation without beginning
a Component parse. A parsed body that refuses text custody remains normally
owned; no filtered diagnostic or fake completion replaces it.

The same allocator/source lifetime is passed directly to the splitter and
the Component parser. Whole-root fat-pointer extent, selected content pointer,
byte start/index and original name spans come only from the private splitter
selection. No allocator address, numeric identity or equal source contents
can mint custody. Moving the observation before borrowing preserves the
actual owned Component; an existing view prevents owner move/drop. A second
same-buffer parse is still a different Component, and the existing child
join must reject its child. The design promises physical source custody,
not source allocation identity beyond the actual retained borrow.

## Sole construction and minimal source seams

Add `container/vue/descriptor/vue2.rs` and its owned `vue2/tests` siblings.
Only a narrow module/export/entry seam belongs in `container/vue.rs` and
`descriptor.rs`. The current files are 334 and 323 lines respectively;
keep their existing source-size ceiling by using the new sibling module.

Reuse the existing private policy and template classification at the same
splitter callback. A small private initializer helper may take the required
version: existing `Policy::new` continues to require V3, while a separately
named Vue 2 constructor requires V2. It must not make V2 accepted by the
existing public `observe_descriptor` entry. A new private Vue 2 callback
rejects script/style/custom roles before invoking the existing template
record method. Proposed precise script/style issue codes are additive and
unused by the V3 route; no new policy mode field is required.

Run the existing `split_with` exactly once and finish all envelope checks.
Only after successful final selection call the existing
`vue2::surface::parse_component_block` exactly once with the selected block.
Retain that returned owner directly. No second splitter, lexical source
search, substring parse, CST walk, filter scan, decode, serialization or
pipeline stage is added. The existing interpolation callbacks retain their
sole original expression parses. There is no resumable public construction
or retry path; an unwind cannot return an admitted observation.

No existing File row, ScriptUnit, child layout or parser receipt changes.
The new observation's Container/issues/CST storage is real added ownership,
not a zero-memory claim. Inventory its actual fields and existing parser
storage honestly; never put ComponentParse or NativeSyntax in an arena Vec
that would skip their normal diagnostics destructors. Every old layout,
allocation and instruction gate remains unchanged, including all 104
protected benchmarks. Existing measurements alone do not prove this new
entry's parse count, storage footprint or whole-source construction cost.

## Meaningful hosted laws required after authorization

- Positive whole-source empty/static/interpolation/filter templates, optional
  `lang=html`, HTML comments and Unicode/nonzero prefixes; original root/content
  fat pointers, block index, opening/content/closing geometry and complete
  CST fidelity must agree. An admitted filter read checks the existing actual
  AST, genuinely empty comment/diagnostic records and authored maps through
  its genuine TextView; it never manufactures a comment-bearing admission.
- Original splitter diagnostics and all profile/block/attribute/boundary
  refusals above; no Component parse for a failed envelope, including a
  later script/style/custom sibling or duplicate template.
- Recovery/verbatim/encoded-delimiter/argument-hole/body syntax inputs retain
  their normally owned Component and exact existing TextRefusal. Selected
  envelope custody must not silently become clean-body admission.
  Use scanner-complete inputs that reach this envelope: an uncertain or
  unclosed interpolation rejected by `find_template_close` instead belongs
  to the zero-parse envelope controls. Require the actual reachable child and
  `text_for` result; never manufacture an interpolation child after refusal.
- Expression `//` and `/*` controls retain the original Component/binding
  `CommentSyntax` boundary and require the actual TextView refusal. The current
  Vue 2 scanner refuses them before admitted chain parsing; do not invent
  a started NativeSyntax or comment-bearing TextView for those inputs.
- Same-buffer second parse, equal foreign source allocation/arena, sibling
  child, original parent/ordinal and moved owner controls. A foreign child's
  existing `text_for` join refuses even when bytes/spans match.
- Correctly typed private-constructor compile-fail laws, no modern-carrier
  conversion, view/owner move and drop, source/allocator escape and non-Clone
  controls. A private fault hook at the sole parser call proves no retry or
  returned completion after caught unwind without adding production storage.
- Preserve all V3 Descriptor sources/refusals and modern selected laws;
  existing Vue 2 custody/filter/framing/argument and complete pinned 2.7.16
  compiler/dev/prod reference laws stay unchanged. Add no legacy normal edge.

Count proof uses test-only observation at the actual existing splitter and
parser entry, rather than a source-length proxy or added runtime counter.
Its exact hook placement and storage must be peer-reviewed before source
freeze, with per-test/thread isolation against concurrent law contamination.
Automatic exact-head Actions establish actual compilation, laws,
doctests and inventory; protected full suites/unchanged 104 gates and actual
merge would be separate later acceptance, not a private-plan credit.

## Independent work and remaining gates

The Vue 2 formatter lane reserves only Glyph's borrowed expression origin,
`native_doc/vue2_text` and its consumer laws/fixtures/oracles. It consumes
the already authentic TextView directly. This descriptor design neither
blocks that independent work nor grants it selected-SFC completion.
Completion audit reserves no descriptor edits. Root owns the implementation
and publication decisions; retain existing worktrees and cutoff constraints.

Historical File/whole-chain legalization, JS lifetime attachment, ordered
child completion, bindings/filter registry, classic runtime, full code/maps,
product/history/default replacement and V2_7/other dialects remain unfinished.
No roadmap or fix-history issue closes from this structural owner. This
plan and the local paired issue draft are frozen for root and retained peer
review before any production edits.

## Retained peer clarification

Completion audit's full read-only review of `82bd09e11f` is design CLEAR,
with actual test-hook placement/counts, later-sibling no-parse and normal
diagnostic drop/unwind laws still required at implementation freeze. The
retained dialect peer identified ambiguous positive-comment law wording:
the actual Vue 2 scanner refuses expression comments as `CommentSyntax`.
The plan now distinguishes HTML-comment positives, genuinely empty comment
records on admitted chains, and original expression-comment refusals. This
also distinguishes zero-parse splitter refusals from scanner-complete body
refusals with actual reachable child custody. These are law-premise
corrections only, with no implementation or execution credit.
