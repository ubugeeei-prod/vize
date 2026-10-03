# Original JSX/TSX checker projection

Related: [#6840](https://github.com/ubugeeei-prod/vize/issues/6840),
[#6849](https://github.com/ubugeeei-prod/vize/issues/6849),
[#6879](https://github.com/ubugeeei-prod/vize/issues/6879).

The original owning JSX File providers and JS runtime module target already
actually merged. The checker projection still refused every JSX profile.
Add specialized `project_jsx_program` and `project_jsx_program_no_links`
entries that accept only the genuine completed `JsxFile` owner. The result
borrows that owner's same File, keeping the actual original observation alive.
A separately supplied File or AST cannot mint this JSX entry.

Retain every existing complete-File, sole-unit, whole-root source-span,
top-level scope, empty neutral template tree and source-size check. A genuinely
completed partial-block JSX owner still refuses with `PartialSource`.
The generic original Program entries continue refusing JSX profiles.
The private emission mode is selected only by these concrete owning entry
points; no public caller flag or custom link sink can authorize it.

Copy the complete original source once through the existing linked writer,
followed by the existing unlinked Module suffix. JSX remains JSX and TSX remains
TSX, with distinct `SourceKind::Jsx` / `SourceKind::Tsx` extensions.
Every original byte, comment, directive, hashbang, entity spelling and type
annotation admitted by the actual sole lower walk remains intact. No parser,
AST/node traversal, semantic walk, context globals, JSX factory or TS erasure
is introduced.

The existing authored covering link and `map_span` / `map_utf16` APIs retain
exact source ranges and typed surrogate/generated/boundary refusals. The
serialized map retains the existing whole-source start anchor and complete
original source content; this does not claim a complete token/line-anchor map.
Recorded and no-links bytes are identical, and no-links diagnostic mapping
refuses explicitly.

Four authentic laws cover original owner moves, exact JS/TSX complete source
and map fields, UTF16 with Unicode and CRLF, generated suffix refusals, genuine
partial ownership and original lower profile/parser-hole/incomplete refusals.
Compile-fail laws prevent separately supplied File admission and dropping the
owning body while its projection lives. Local static/rustfmt/assertion checks
are complete; no local Cargo build or backend run supplies acceptance.
New exact-source Actions and protected full/all-100 remain required.

The two existing Canon source-kind matches add explicit JSX/TSX refusal arms
only to remain exhaustive. The actual configured JSX checker consumer is a
separate genuinely dependent slice. Runtime TSX emission, broader original
lower syntax, unresolved component inputs, full fix-history comparison and
default replacement remain unfinished. #6849/#6879 remain open.

The preceding original Program diagnosing-project correction #7617 actually
merged at 2026-10-03 16:54:26 UTC as signed
`8107bca95833da6fe2d0885bb01fb86e14d44f4e`, verified in fresh main.
Source989c97e3 Check37136262577 and protected Check37137606247,
Musea37137605850 and Nuxt37137605848 all succeeded. The protected full
workspace archive, doctests/differential corpora, four actual Rust workers and
source report passed. All ten original complete vectors, the genuine late
project/root-config refusal and reaping controls, inherited-option reload,
original timeout cleanup and all nine native Vue laws ran successfully again.
All100 benchmarks were identical across three executions and passed unchanged
ceilings/ratchet (instruction job111245211451, artifact11279710647).
This new L4 provider replays onto that literal main; it inherits no pending
Canon source or historical acceptance.
