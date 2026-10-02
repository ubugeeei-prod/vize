# Vue interpolation source preparation

Paired issues: [#6836](https://github.com/ubugeeei-prod/vize/issues/6836)
and [#6840](https://github.com/ubugeeei-prod/vize/issues/6840).

The native construct provider prepares the exact parse/emission source before
its sole language parse. `prepare_vue_interpolation_in` receives the compile
arena, authoritative file and existing delimiter-free authored content span.
It checks the full file/range, selects only TAB, LF, FF, CR and SPACE at the
authored edges, then decodes the selected window once in HTML **text** context.
The result is the existing opaque `EmbedSource`, with narrowed authored span,
complete decoded text and the native private checked correspondence map.
Reference-free and unknown-reference text remains borrowed without allocation.

This follows the pinned Vue 3.5.35
[interpolation parser](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/parser.ts#L105)
and its [five-byte whitespace predicate](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/tokenizer.ts#L141).
Trimming decoded text would incorrectly remove entity-produced whitespace;
selecting the AST/comment minimum would also remove authored non-HTML trivia.
Directive values have a different
[whole-value policy](https://github.com/vuejs/core/blob/v3.5.35/packages/compiler-core/src/parser.ts#L338).
The existing public attribute helper therefore retains its span and attribute
decoding context; both entry points share the same private once-only decoder.

The caller must keep the Artifact's authoritative file and full interpolation
op/provenance span separately. The prepared source alone is narrowed. It goes
directly into the existing `parse_once` and safe consuming handoff, preserving
the parsed AST payload/children, all original comments/diagnostics, parser
prefix and checked source maps. The native L1-to-L2 bridge then retains that
exact handoff root pointer; no second parser, extra pipeline stage, serialization
or independent output-window field is introduced. L4 continues to emit the
complete expression source, with no trimming, searching or alignment recovery.

This helper supplies source preparation, not backend or language admission.
NBSP, BOM and vertical tab are retained rather than treated as HTML whitespace.
Vue's NBSP/BOM simple-name prefix positioning remains a separate dialect
spelling gap: the actual native interpolation hook must return a typed refusal
for that unsupported family, with original source/observations retained.
No native byte-parity credit is assigned to those source-only preservation laws.

Line-comment termination is actual source data. A trailing authored LF is
removed before parsing; a trailing `&#10;` produces a retained LF after decoding.
The existing L1 expression wrapper contributes its own private newline, so a
successful wrapped parse alone cannot prove an emitted trailing comment is
safe. Leading/trailing `//` remain refused by the unchanged neutral retained
handoff trivia guard. Internal line comments keep their real newline. The
helper does not loosen a shared legacy guard or remove bytes after parsing.

## Evidence and limits

Nine ordinary integration laws compile the actual whole L1 source and use its
native decoder/parser/handoff: five-byte-only selection and zero-allocation
borrowing, retained non-HTML trivia, text-versus-attribute ambiguity, once-only
decoding, complete entity whitespace/operator maps, multi-scalar partial-edit
refusal, all retained block comments and stable parsed payload addresses,
actual line-comment newline observations, and invalid UTF-8/empty-window refusal.
Entity laws compare complete authored lexemes; line-comment laws compare the
complete retained tail, including its newline and following source bytes.
The AST enum root moves into the shared arena during the existing handoff;
its original allocated payload/children and comment text addresses survive.
The laws do not claim that the pre-handoff enum address remains unchanged.

Direct scoped Clippy passes for the actual whole L1 library and this test under
the workspace panic/indexing/todo policy. Checked range/projection endpoints
reject invalid UTF-8 and entity interiors; diagnostic covering projections
cannot select output edits. The selected map covers all emitted source bytes.
No new owned storage site or dependency is added. Generated source inventories,
storage laws and exact-head Actions precede publication; all existing
instruction/allocation ceilings and protected queue gates remain unchanged.

TODO: the real native interpolation consumer must call this helper before
`parse_once`, keep the full construct identity/source/provenance, and report
the separate typed spelling/admission holes. Complete native DOM module/maps
must be verified through that actual provider, not caller-rebuilt expressions.
This helper alone does not implement that consumer or a complete native
compiler. Full grammar, every Vue dialect, file binding/script declarations,
backend completion and product integration remain unfinished. #6836, #6840
and the compiler fix-history gate #6880 remain open; legacy routes are unchanged.
