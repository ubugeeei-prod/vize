# Generic component prop hover preserves declaration types

Issue: [#8015](https://github.com/ubugeeei-prod/vize/issues/8015).

The canonical generic-child prop navigation anchor can return the exact native
signature `unknown`, while the current component interface retains `T[]` and
`string`. Prop hover previously replaced its declaration signature with that
uninformative native quick info.

Keep concrete native signatures and their complete documentation unchanged.
When native markdown is exactly `unknown`, or its TypeScript code fence contains
only `unknown` or a property signature whose complete type is `unknown`, show
the existing current declaration signature. Preserve the complete documentation
after that fence and the native/authored range. This changes only prop hover
presentation; event hover and all provider requests, readiness, navigation,
diagnostics, caches and pipeline stages stay unchanged.

The differential fixture retains the complete original issue and its four SFCs
with SHA256 custody. Three Rust laws compare complete original prop hovers and
exercise concrete native types, optional types, unions, nested unknown members,
`any`, unknown prose and complete documentation. A source-built stdio law uses
the original four-file Vue project, compares both whole hover payloads and
ranges, then checks an unsaved Unicode/CRLF edit, unchanged disk and restored source.
The existing native-phase Actions cell explicitly runs this CLI law, alongside
the ordinary source checks and protected full suite.

The issue accepts declared `T[]` when native instantiation is unavailable. This
slice satisfies that fallback; it does not claim a new instantiated native prop
type. Common component attributes are owned by #8131. Expression globals and
additional data/ARIA candidates remain separate work. Keep #8015 open until
all original acceptance cases are delivered. Exact-head Actions, protected
instruction/full gates, signed merge and public delivery remain pending.

The first source/native runs rejected SHA256 array LowerHex formatting in the new custody test before execution; use the established byte-by-byte hexadecimal renderer. The native workflow reached352 lines after two added paths, so replace the previous exact CLI path with an inclusive LSP CLI pattern and retire the now-redundant computed-inlay exact path. All old triggers remain covered and the workflow stays350 lines; production and all original/expected payloads are unchanged. Fresh successor Actions are required.
