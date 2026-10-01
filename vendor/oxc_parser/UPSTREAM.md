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
