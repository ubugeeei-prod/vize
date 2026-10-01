# Pinned OXC PURE error recovery and comment rewind

Tracked with the actual native syntax work in
[#6836](https://github.com/ubugeeei-prod/vize/issues/6836).
Normal Program admission exposed a genuine pinned-parser failure; parser
defects are repaired rather than hidden behind input rewriting or comment skips.

The pure source-copy commit preserves exactly 53 official blobs, 1,023,751 bytes, from
OXC `0.142.0` commit `fc702c1fa9f0412d06ec6908b58cd395b826cf7f`, including its
original MIT license. The source-copy commit is separate from manifest and
behavior changes. `tools/support/dependencies/vendor-oxc-parser.ts` reproduces
the unmodified source directly from verified Git objects, ignoring dirty
checkout bytes. `UPSTREAM.md` records the source and license identity.

Only two sites in `lexer/trivia_builder.rs` change:

- Error recovery can mark the same PURE annotation twice. `PureNotApplied`
  is a valid terminal state, so the debug invariant admits it alongside `Pure`.
  The assignment remains idempotent and this edit alone has unchanged release
  behavior. Unrelated comment kinds still fail the debug invariant.
- Duplicate-comment rewind must recover the original comment index by sorted
  start offset and matching full span. The old `len - 1` index could point to
  a later ordinary comment and silently retag it in release builds.

The first cause is documented by official
[PR #24161](https://github.com/oxc-project/oxc/pull/24161), using the bare-colon
reproducer from [#23669](https://github.com/oxc-project/oxc/issues/23669).
Official commit
[d5163d0f](https://github.com/oxc-project/oxc/commit/d5163d0faac7c936796dd5c0164d58ab5eae92ea)
includes the exact-span index correction but also rewrites annotation
application more widely. This narrow local repair preserves the pinned public
parser/API and avoids that broader dependency upgrade.

The standalone manifest normalizes upstream workspace inheritance to explicit
original package metadata and dependency versions. Every supporting OXC crate
keeps the official exact `fc702c1` source; AST/allocator/span identity remains
shared with Vize. Root and isolated fuzz Cargo workspaces use the same Git-source
patch. The third-party crate is excluded from Vize workspace membership, uses
its upstream license and does not inherit Vize's own lint/storage policy.
Existing vendor handling in the source-length gate is unchanged.

Actual offline Cargo metadata for both workspaces proves one parser and one
official supporting OXC universe. The root lock change removes only the
parser's Git-source line; offline locked metadata passes. No allocator, extra
parser pass, serialization, normal legacy edge or instruction ceiling changes.

`upstream_pure_comment_assertion.rs` now directly replays every formerly
excluded expression boundary, with ordinary returned-diagnostic drop and no
panic catch. It separately checks that Program rewind retags only the original
PURE annotation. The expression fuzz target no longer skips PURE annotations.
`tests/fixtures/vdom/pure-comment-recovery.pkl` adds real differential compiler
cases for annotation/ordinary-comment ownership.
The active differential coverage list includes both cases. Their complete
expected compiler bodies and digest receipt come from actual Vue
`3.6.0-beta.10`, independently of Vize's original or corrected parser.
`tests/tooling/support/generate-oxc-pure-expected.ts` reproduces that oracle.

The bounded `Pinned OXC PURE Capture` manual workflow builds actual original
and corrected optimized parser/OXC printer/legacy compiler observations on
the same head. The original job restores the exact reviewed upstream trivia
Git blob, verifies its object identity and records the source diff; it does
not generate an expected baseline from candidate execution. Artifacts retain
source/profile identity, whole AST and diagnostics, OXC printed output, and
legacy parse/transform diagnostics and complete generated compiler bodies.
These hosted comparisons must be reviewed before queueing.

The original hosted ZIP artifacts remain authoritative and retain their GitHub
digests. When that download route is unavailable, the same opt-in job exports
every captured file losslessly through `export-capture-evidence.ts`: sorted
relative paths, exact base64 bytes and per-file SHA256 are packed into gzip.
Framed logs retain both raw/gzip lengths and SHA256, sequential chunks and an
end digest; the exact gzip is also uploaded separately. Admission is bounded
to 128 regular files, 16 MiB of captured bytes and 2 MiB compressed. Symlinks,
output within the input and exceeded bounds fail rather than truncate output.
The exporter runs after observation and changes no parser or product pipeline.

Nix's dependency build retains only the real reviewed `vendor/oxc_parser`
through Crane's `extraDummyScript`; Vize sources remain dummyized and cached.
Official OXC crates need the patched parser's actual public API, which a dummy
local `lib.rs` cannot supply. Supporting identities, full Nix gates, TOML
formatting and generated consumer inventories remain enforced.

The reproducible Rust observer and TS replay under
`tools/support/dependencies/` capture complete actual Program ASTs, comments,
owned diagnostics, parse status and exact input/profile identity. Fixed debug
and optimized release parsers complete 16,496 combinations of annotation/
statement/comment variants and every source prefix in JS/TS/JSX/TSX,
Module/Script. The assertion-only repair is byte-identical to the original
release capture. The complete correction changes 256 observations, solely
comment classification; AST structure/flags, spans, diagnostics and parse
status are unchanged in that bounded matrix. Original and candidate raw
captures and source identities are retained independently of Cargo outputs.

Locally, five actual legacy parser regressions pass. The approved ordinary
Program provider's current L1 source passes its scoped unit/AST laws with the
same private corrected parser. This is not full-product byte parity or
exhaustive parser safety evidence. Exact-head Actions must pass ordinary/full
checks, legacy output/formatter suites, strict100 and restored/native fuzz
before protected native Stack queueing. Publication and queue remain root-owned.

TODO: continue the genuine dependent ordinary Program provider and explicit
standalone script inspection consumer; retain separate wrapped-shape admission,
file language/container/identity integration and measured parser hardening work.
The issue remains open, and native end-to-end product completion remains unfinished.
