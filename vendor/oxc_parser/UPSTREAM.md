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
