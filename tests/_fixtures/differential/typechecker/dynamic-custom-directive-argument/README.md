# Dynamic custom directive arguments

Element Plus at `20298a21ae54a5417955373b80130ae7c548a6bb` uses
`v-clickoutside:[contentRef]` in Cascader and
`v-click-outside:[popperRef]` in Select and Select V2. These DOM element values
are Vue directive binding arguments, rather than JavaScript property names.
Vue's actual `DirectiveBinding<Value, Modifiers, Arg>` defaults `Arg` to `any`.

The minimized complete project keeps both original directive spellings,
unwrapped element refs, and the genuine Vue binding/value hook. The native
law compares the whole diagnostic vector for the legal project, an unknown
argument, an invalid argument member, a wrong handler value, and each repair.
The original built-in spread/dynamic-name native laws remain required.

Classification uses the parser's directive name and exact argument spans in
the existing AST traversal. It does not infer a directive from source prefixes,
remove argument expressions, filter diagnostics, or weaken hook value checks.

The three original upstream files and their pinned blob identities are listed
in `original-custody.json`; no upstream changes are made. A green minimized
project does not replace the frozen real-project matrix's full 224-case gate.
