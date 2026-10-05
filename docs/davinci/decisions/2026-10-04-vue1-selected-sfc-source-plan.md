# Original Vue 1 selected SFC and formatter source plan (2026-10-04)

Design only on actual main `76f93fe1e3c46ac8c86157a520302e26c50ac871`.
No implementation, native execution, publication or acceptance is supplied here.
The separate #7777 source `f3bb6ac4d08b773311194175b124bb8575c72357` is
ready/source-green and independently reviewed, but its actual merge is pending
fixed `v0.432.0` release sequencing. This plan never delays that delivery.

## Available authority and the precise gap

Actual `dialect/vue1/surface.rs` already owns one normal `ComponentParse`, its
complete-root `SourceBlock`, errors, unsupported framing, original bindings,
text boundaries and normal Drop. `surface/body.rs` supplies nonclone original
`TextChild`/`TextView` membership, real parent/ordinal/node pointers and the
once-observed NativeSyntax/AST. No new callback provider or observation is needed.
Actual `container/vue/descriptor.rs` admits V3; its distinct `vue2/*` owner admits
V2. Neither grants V1 selection. A raw Container, copied tuple, modern Component
or a caller-paired body/source must never mint selected Vue1 authority.

The genuine checked Vue1 TextDoc prerequisite is #7777, not an actual-main API
at this plan's freeze. It keeps Doc private and refuses every CrLf print option.
Existing Glyph `FormatResult` is exactly code/changed. The printer has no generated
VLQ map contract. Required maps below are the complete real original/prepared
callback maps and whole-root spans; generated printer maps remain unclaimed.

## Small independent L1 provider

Add sibling `descriptor/vue1.rs` plus private `vue1/{observe,view,tests}.rs`,
exporting `Vue1DescriptorObservation`, `Vue1TemplateView`, `Vue1DescriptorRefusal`
and `Vue.observe_vue1_descriptor(allocator, source, DescriptorOptions)`.
Normal owner fields are private: original Container/root/options/issues,
selected frame and optional actual `dialect::vue1::surface::ComponentParse`.
Use explicit V1/Vue/default SurfaceParseOptions; retain every original refusal.

Reuse the existing `split_with` block emission callback and narrow existing
Policy version constructor. The same block event rejects every script (including
empty/setup), style, custom/external block, nonliteral spelling, ambiguous or
encoded language, duplicate/missing template, recovery, self-closing selection,
unsupported options/version/dialect. At the same emitted block event, an O(1)
`Block.attrs.is_empty()` check refuses all outer template attrs without revisiting
its header.
Do not rescan headers, walk a Container again or reinterpret selected capture rows.
Only after the complete splitter result is clean, check actual root/block/name
slices and call Vue1 `parse_component_block` once on that original selected block.
Park the complete normal owner, including body errors/holes, before returning.

`selected()` creates a short view of this exact retained owner/component, actual
index/block/opening/closing name spans and root-fat-pointer identity. It grants
an envelope, not clean body/grammar/runtime completion. Repeated reads reborrow;
no parse/decode, source factory, AST walk, detached Clone, conversion or public
constructor/from-capture tuple is available. Body diagnostics stay separate from
splitter/policy diagnostics, including when selection succeeds with a syntax hole.

Provider laws require real entry counters in the existing splitter and V1
Component entry, exactly once/zero before refused envelopes; repeated readback,
normal Drop and caught entry/after-park unwind, concurrent thread isolation,
nonzero Unicode root/UTF8 slices, original options/profile/diagnostic vectors,
foreign same-buffer owners/equal-byte buffers, move-before-borrow, and private
constructor/lifetime/no-modern-conversion compile-fail laws. No extra stage or
serialization; existing V2/V3 entries and all old vectors remain byte-exact.

## Immediate whole scriptless Glyph consumer

New independent siblings `native_doc/vue1_sfc.rs`, `vue1_sfc/{build,observation,
options,refusal}.rs`, and `vue1_template.rs`/private header/tests. Only tiny module
registration and a private genuine Vue1TextDocument handoff are shared seams.
Do not edit pending Vue2 files, its header policy, Expression Origin/Context or
its capture action. Existing actual token Cursor can check original slices.

`observe_native_vue1_sfc_in` owns the real descriptor/options and private
Doc-or-refusal outcome, with selected views reborrowed on demand: no self-reference.
Require genuine selected owner and original complete clean body; refuse recovery,
raw/once/all-pipe/framing/encoded-boundary/native-hole and unsupported expression
forms using existing earliest typed APIs. The first static family has one actual
root Element among div/span/p/section/a/br/input, allowing only original outside
whitespace/comments. Root interpolation/nonwhitespace text refuses in this same
child visit; Vueify outer template-count validation certifies neither this body-root
policy nor a required template. Allow original inside text/comments and nested
same-owner children. Empty/multiple-root/unrecognized element and every authored
attribute refuse. This is deliberate: V1 interpolates decoded attribute values;
V2 plain-header admission cannot authorize V1 attributes without a real provider.
Depth is bounded by the existing consumer limit, with no semantic credit on refusal.

One existing child/token consumption builds the body Doc. Each interpolation
joins its actual parent/ordinal/surface/component and full callback span to
`component.text_for(child.reborrow())`, then the genuine checked Vue1 TextDoc.
A private handoff may consume that sealed wrapper into original-view + Doc only
inside native_doc; verify same original occurrence and append to this one Doc.
It never prints a callback separately, reparses/decodes or walks the AST/body again.
The same actual traversal can append dev-only original receipt rows; no production
per-node index or capture pass is added.

Join original descriptor/component/block/name membership and actual whole source,
then construct authored prefix + body Doc + authored suffix before the sole printer.
Complete LF output includes untouched outer framing and prefix/suffix in lookahead;
no post-print splice/trim/fallback. Public owner has source/options/descriptor,
short selected view/refusal inspection and checked format returning code/changed.
Doc and into_parts are private: no document accessor can bypass the unconditional
CrLf refusal, even for flat atoms or sources lacking interpolation. Refusals retain
actual normally owned observations and expose no partial successful output.
Owned formatted String may outlive owners; wrapper/source/arena authority may not.

## Independent original full-envelope and execution proof

The current `sfc-baselines.ts` explicitly returns unsupported for dialect1.
Pinned Vue1.0.28 exposes stock callback parsers/getters and compiler.compile;
it has no full SFC parseComponent entry. V2 parseComponent can be a separately
labelled structural comparison, never Vue1 envelope or semantic authority.
The actual Vue1-era full-SFC source is [vueify v8.7.0 compiler.js](https://github.com/vuejs/vueify/blob/6d08c98bf1e3db6a866b8a0a5f4fa2b5d7131926/lib/compiler.js):
real parse5.parseFragment with locations, validateNodeCount, processTemplate,
parse5 serialization, de-indent, warnings and actual test/production processing.
Its package requires parse5 ^2.1.0; that range is not a reproducible runtime pin.
Before execution, freeze exact registry tarball/commit/files/license and all real
transitive parser/serializer/de-indent/validator/minifier dependencies, native
Node/executable/source hashes, inputs/options/env and full raw process reports.
No local install or new runtime campaign is authorized by this design.

Freeze whole independent input/print vectors before implementing the consumer:
eight original source families at widths1/12/80, indent2, LF (24 rows): direct
binding; ordered call/binary; member; nested/repeated callbacks; physical LF;
Unicode constant/entity literal; accepted complete block comment; nonzero Unicode
outer comment/framing. Require the complete independently fixed code/changed,
then reobserve/reprint that actual whole LF SFC with identical options as a fixed
point. Add width/indent/CrLf refusal controls without manufacturing successful output.
Every accepted family must earn its own full-envelope and runtime proof; otherwise
retain its exact original input as a typed boundary/control, never substitute it.

For each original and formatted whole source independently invoke the pinned real
Vue1-era full envelope path; retain actual fragment locations, selected serialization,
de-indent result, complete warnings/errors/process output and actual emitted module.
Observe every stock callback in source order through pinned Vue1.0.28 test/production
parsers/getters; preserve full tokens/body/value/errors/own descriptors/stacks/noop
and ordered getter/call traces. Equal undefined/noop never earns semantic credit.
Constants may have no calls; effectful controls must have nonempty independently
fixed ordered traces. Preserve entity normalization/browser preparation distinctions.

Use actual supported Chromium, original Vue1 distribution and real full template
mount/clone contexts for original versus formatted independent modules; compare
complete actual DOM/text/attrs/comments/warnings/effects after mount, mutation,
nextTick and destroy/removal, with separate fresh test/production contexts. No
handwritten compiler, getter, decoder, browser DOM surrogate or mocked success.
Retain original class/attribute interpolation, raw/once/pipe/CR/CRLF/U2028/U2029,
empty/multi-root, invalid getter/noop/throw and scripts/styles/custom/external/profile
inputs as complete earliest typed/no-credit controls with original raw owner packets.

Hosted capture must save the complete source-built Rust packet and full primary
process/runtime packet BEFORE fallible comparisons and upload both on failure.
Missing packet, unknown base, wrong source/execution/run/executable/fixture pins,
missing/extra/reordered rows, incomplete maps or unreviewed expected output fails.
Capture includes complete whole source/code/changed/root/template/callback spans,
actual original/prepared texts and complete decode-map segments for every callback,
original component/slot/refusal evidence, formatter fixed point and primary hashes.
Keep original #7777 64/new12/nineCF+positive, old48V2 and sevenV2/fiveV1 packets;
add only a distinct whole-V1 action after the delivered neutral history composite,
at the existing guarded first shard/base, retaining workflow350 and all other hooks.
Current source-only review, compiler-only observations, browser proof, protected104
and actual signed merge are separate receipts; all new execution remains unprovided.

## Delivery and unchanged scope

Root and retained peer must review this frozen design before implementation.
The L1 provider is independent; its consumer is a genuine child from that exact
provider head in a native GitHub Stack. #7777 is a real consumer API prerequisite,
not a fictional dependency imposed on the L1 provider. Integrate only actual main
or explicitly reviewed genuine private dependencies; record native Stack order
and source/candidate/literal receipts when publication is authorized.
Pair future decisions on #6837/#6842 and the formatter history issue #6882
(currently closed, not reopened or promoted into full Vue1 proof here) with the
central record. Existing legacy/default APIs, history/default migration and full
Vue1/Vue0.x/quirks grammar/runtime remain outside this first bounded admission.
