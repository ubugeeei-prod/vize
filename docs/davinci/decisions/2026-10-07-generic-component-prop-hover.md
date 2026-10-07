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

The clean owned #8162 refresh rebases onto actual main
`c86c7a21f9b858122c0aa42f9771555e33a87c32`. All eleven owned source/control
files retain their exact `494944d350` bytes, and all incoming attribute rules and
canonical entries remain intact. The canonical record and native workflow each
retain 350 lines. Previous original stdio and native-signature successes remain
historical receipts; fresh exact-head source/native Actions are mandatory.
Admission remains held for corrective #8181 and verified release delivery.

The next reviewed existing-PR replay incorporates actual signed main
`3ed1cc90908c88016310b90a855500c94a156f1f`, including the actual corrective
#8181 and project-context #8140 merges. All eleven owned source/control blobs
remain byte-identical to the previously qualified `320c98688b` head. Preserve
main's model-modifier native target/filter and compiler/formatter corpus
attribute rules alongside the original generic-hover target and triggers.
The root-approved common canonical union retains its exact 350-line bytes;
prospective analysis objects supplied only decision prose and never source
ancestry. Previous 16,550-test, original stdio, native and corpus outcomes remain
historical until this genuine union's fresh exact-head Actions pass. Root owns
finite queue admission, protected qualification, actual merge and release
delivery. Contextual data/ARIA completion and combined #8015 delivery remain
unfinished.
