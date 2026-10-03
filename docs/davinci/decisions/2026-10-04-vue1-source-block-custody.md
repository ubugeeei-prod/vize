# Original Vue 1 complete-file source custody

Paired issue: [#6842](https://github.com/ubugeeei-prod/vize/issues/6842).
The structural prerequisite also remains tracked under
[#6841](https://github.com/ubugeeei-prod/vize/issues/6841).

## Decision and authentic provider

The existing intrinsic Vue 1 `ComponentParse` now retains its original L0
`SourceBlock`. `parse_component_block` and
`parse_component_with_authored_block` accept only that pointer-checked source
frame. `block()` returns the same borrowed frame, including complete authored
root, exact selected slice and nonzero file offset. No constructor pairs an existing CST with a caller-supplied source/offset;
the carrier is produced only by this original block parse.

The existing whole-source functions keep their names, signatures and base-zero
behavior. They validate `SourceRoot` once and use its whole block.
`SourceError::SourceTooLarge` retains its public spelling through a re-export of
the actual L0 source-frame error. Source size refusal therefore precedes native
construction, without a second source-size implementation.

Each entry point constructs the existing native Vue 1 lexer, shared scope sink,
event stream and CST exactly once from `block.source()`. Vue 1 callback policy,
intrinsic version and raw-delimiter specialization are unchanged. Both normal
and optional authored recovery trees keep original block-borrowed tokens;
their bytes are located through the same retained full-file frame.

## Observation coordinates

The lexer and Vue 1 dialect policy produce the same local observations during
that construction. The projection translates only the already-owned
`SurfaceError` offsets and `SyntaxBoundary` spans by `block.start()`. It does
not inspect the source, events, either CST, headers, embedded expressions or
another syntax artifact again. These are the two small diagnostic/boundary
lists already retained by this parser, not a new AST/body traversal or pipeline
stage. The validated block guarantees every local coordinate plus its base is
inside the u32-addressable authored root.

Syntax boundaries now describe complete-file coordinates for raw-delimiter
recovery, empty interpolation, one-time interpolation and historical line
separators. Original markup diagnostics likewise refer to the selected block
inside the full file. Suffix bytes outside that block never become parser
input. Diagnostics do not grant admission, and unsupported text remains in
its authentic byte-faithful tree.

## Laws and acceptance

Seven new native integration laws cover both unchanged standalone APIs/error
spelling; nonzero Unicode roots and authentic raw/attribute token pointers;
all four complete boundary spans; whole markup diagnostic vectors at block
EOF; normal/authored recovery source identity; equal-byte foreign roots and
wrong repeated occurrences; and every UTF-8 recovery prefix with bounded
observations. Two compile-fail examples prohibit the carrier escaping either
its authored root or original syntax arena. The existing distinct modern
carrier compile-fail example and all nine Vue 1 source laws remain intact.

The complete pinned official Vue 1.0.28 bundle and its actual text/compiler
oracle remain unchanged. No license, provenance, reference inventory, legacy
fixture, expected product byte, sibling dialect or shared selected-SFC provider
changes. Existing hosted Rust/tooling/strict checks run this source. Fresh
exact-head Actions, the protected full suites, unchanged all-100 instruction
ceilings, and actual main merge are separate required acceptance evidence.
No local Rust build, test or Clippy execution is used for this slice.

## Unfinished scope

This is original CST/frame custody for the existing bounded component profile.
It adds no Vue 1 embedded AST admission, filters, directive embeddings, browser
document semantics, descriptor selection, runtime lowering, generic SFC route
or default product replacement. Vue 0.x capability tables still have no native
syntax provider. Raw precedence, empty/one-time semantics, historical Vue 1
attribute/filter syntax, quirks, petite and whole-dialect coverage remain
unfinished under #6842/#6841/#6843/#6892. Each product's history gate remains
required before replacing its legacy path. This source-frame prerequisite
does not close those issues or claim native runtime/product acceptance.
