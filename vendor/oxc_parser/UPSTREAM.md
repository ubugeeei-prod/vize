# Pinned OXC parser source

Copied without source changes from official OXC commit
`fc702c1fa9f0412d06ec6908b58cd395b826cf7f` (oxc_parser 0.142.0):
https://github.com/oxc-project/oxc/tree/fc702c1fa9f0412d06ec6908b58cd395b826cf7f/crates/oxc_parser

The adjacent LICENSE is the exact upstream MIT license at the same revision.
Reproduce this source-copy commit with:

`vp node tools/support/dependencies/vendor-oxc-parser.ts VERIFIED_OXC_CLONE`

Following commits normalize standalone manifest dependencies and apply the
reviewed PURE recovery correction. Supporting OXC crates stay at the original
official pinned revision and share one AST/allocator/span identity.

Recursive array expressions and TypeScript tuple types preserve their entire
original parser bodies while using the existing pinned stacker stack-headroom
guard. The tuple guard also covers nested tuple descent reached by expression
type-argument speculation. No grammar, diagnostics or depth refusal is added.

The shared expression type-argument parser also memoizes failed speculative
probes by source position and grammar context across rewinds. This bounds the
nested malformed-generic allocation reported in Vize
[#7421](https://github.com/ubugeeei-prod/vize/issues/7421) and
[#7444](https://github.com/ubugeeei-prod/vize/issues/7444), which affect both
legacy JS/TS consumers and native admission. Exact campaign reproducers live
in `tests/fuzz/regressions/l1_program/type-argument-*.*.input`; legacy TSX
integration tests assert their diagnostic result and arena allocation ceiling.

Failed type-argument and arrow probes share the existing table with disjoint
one-word keys, avoiding another table initialization/drop for ordinary parses.
The type-argument key retains all grammar-context bits.

Vize publishes this unofficial fork as `vize_oxc_parser`, synchronized with the
Vize workspace version. Its Rust library remains `oxc_parser`, and consumers
use a direct `path` plus exact `version` dependency with the `package` alias.
This preserves the admission/observation APIs outside the repository without
relying on a root-only Cargo patch. The fork is published before its consumers;
Cargo replaces the local path with the registry identity for published crates.
Supporting Oxc crates retain the original pinned revision and registry version.
Upstream source retains its own formatting and lint policy; Vize's workspace
continues to compile and test the fork; its crate-level Clippy policy preserves
upstream style without changing first-party check commands.

The unpublished `vendor/oxc_parser_compat` package re-exports this same fork
under the upstream package identity for transitive Git dependencies such as
`oxc_formatter::parse_for_format`. The repository patch retains parser safety
in those paths; published Vize consumers use the direct fork dependency and
do not rely on that patch. The adapter contains no parser implementation.

The original lexer now retains whether it decoded legacy numeric or string
literal spellings forbidden in a strict module. The private boolean follows
both lexer checkpoint/rewind paths and is exposed only from the immutable
original admitted Program observation. Ordinary ASTs, diagnostics and syntax
admission remain unchanged; the receipt does not certify general semantics.
Native setup eligibility consumes it without another source scan or AST walk.

The complete original array-expression body now uses direct pinned
`stacker = "=0.1.25"` with 512 KiB headroom and 4 MiB growth segments for the
authenticated Vize #7805 Program reproducer. Token, grammar, recovery and
diagnostic work stay on the same parser. This guards array descent only;
other recursive grammar paths, unsupported stacker targets and hostile-input
quotas remain separate. The original source is retained in the Program fuzz
regressions and seeded in all eight explicit source profiles.
