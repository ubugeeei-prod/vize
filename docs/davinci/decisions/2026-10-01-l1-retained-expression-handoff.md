# L1 retained expression ownership and checked pieces

Tracked in [#6836](https://github.com/ubugeeei-prod/vize/issues/6836), following
the [typed-embed design](https://github.com/ubugeeei-prod/vize/issues/6836#issuecomment-5847794929)
and the [native shape provider](./2026-10-01-l1-native-embed-shapes.md).

The fused L1→L2 consumer cannot safely keep both a `NativeSyntax` owner and an
expression borrowed from that owner's Program in one returned artifact. The
source tree already exists, so reparsing or cloning it would violate the design.

`NativeSyntax::into_expression` consumes an Expr artifact and moves only the
existing expression enum from the generated parentheses into the shared arena.
All arena-owned descendants retain their addresses, node identities and source
references. No AST walk, second parse, unsafe operation or owner is leaked.
The returned `RetainedExpression` supplies an allocator-lifetime root reference
independently of its observation owner's lifetime. A consumer can retain that
same root reference and drop the observation owner normally.

Comments retain the same original arena slice. Complete OXC Diagnostics stay
in an ordinary owned field with a normal destructor; they are never put in
drop-free arena bytes. Syntax/admission holes preserve source, comments and
diagnostics while exposing no recovery AST. Wrong-shape artifacts are returned
intact in `alloc::boxed::Box<NativeSyntax>` rather than silently discarding their
Program/body/list views. Only this rejected-shape path adds a cold heap
allocation; accepted expressions move their root into the existing shared arena.
The boxed error keeps ordinary destruction and a small error return without any
Clippy exception. No workspace lint or storage classifier is relaxed.

The handoff exposes the actual parser-prefix metadata and checked
decoded/authored projections. L2 must consume this metadata, rather than assume
the Expr wrapper always contributes two bytes. Its neutral coordinate adapter
may copy checked segment records, but must neither decode again nor replace
the retained root with a parsed text approximation.

`EmbedSource::slice_in` selects a UTF-8 decoded-relative piece and projects
its exact file-absolute authored range through the existing checked map.
Whole entities are indivisible; an interior cut or point remains
`PartialEntityBoundary`. Derived maps retain complete entity atoms and shorten
only validated identity segments, with checked authored additions. Their
decoded ranges are relative to the new piece, while authored offsets remain
file-absolute. A full slice reuses the parent's map; empty and identity-only
pieces borrow source without allocating a map. No HTML reference is decoded
again, including a decoded spelling that itself looks like `&amp;`.

Five consuming-handoff laws cover live roots after owner drop, preserved root
children/comment/diagnostic pointers, authored parentheses, decoded source maps,
failure observations and intact unsupported shapes. Six source-piece laws cover
UTF-8/file offsets, clipped identities/complete entities, nested pieces, map
reuse and allocation-free identities/empty pieces, invalid boundaries, no
second decode, and every subrange's exact/covering projection against its parent.
All 46 inherited and new native laws pass in the scoped actual-source `no_std`
rustc harness with existing pinned OXC libraries. The same actual modules pass
strict Clippy with warnings and wildcard imports denied. This is module evidence,
not full current-workspace Cargo/Clippy/Actions proof. Exact-head Actions, full
performance gates and protected queue delivery remain required.

TODO:

- Connect the actual neutral L2 retained-expression consumer without reparsing.
- Use checked pieces for dialect-built ForHead and FilterChain artifacts;
  preserve sparse binding positions and distinguish actual argument lists from
  slot bindings or sequence expressions. Whole composite text is not JS Expr.
- Keep full grammar admission, JSX/TSX, file language/identity attachment and
  native product-route gates unfinished until their own providers and parity pass.
