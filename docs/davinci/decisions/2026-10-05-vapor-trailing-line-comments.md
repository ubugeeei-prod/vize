# Vapor expression line-comment boundary (#7894)

The reported original App.vue is retained exactly: 138 bytes,
SHA-256 b7f2f1bea9c5157e0473ea690adf6a9572c6ba86536d177cac50936b4f41408f.
Its reporter is GitHub ubugeeei, verified public ID 71201308.
The implementation starts from actual main ef506012169d228982ded40d893ac2115a3e58a7.

Vapor trims expression trivia, then callers append closing punctuation.
A final line comment consequently swallowed that punctuation and broke the
complete module. End the parse-only parenthesis wrapper with a newline,
keeping identifier offsets exactly one byte, and convert actual line comments
after all span rewrites with the established DOM comment converter. The
retained differential uses that same parse boundary and rewrite contract.
The shared lexer must recognize full Unicode identifiers and keyword-named
properties before deciding division versus regex; an independent source peer
found a quoted-slash corruption in the initial reuse, before any acceptance.
Preserve quoted strings, regex, template literals and comment-free output.

The corpus contains the original plus fourteen fixed controls. It covers
reactive if/else-if/show/title/class, same-line comments, a block terminator
inside comment text, Unicode/property division with trailing comments and
quoted slashes, and working event/html/regex-string controls. Source-built
full SFCs run in development and production: 30 complete public results,
30 independent whole-module trace pairs, each with three rendered states,
exact diagnostics/interactions and complete unmount. Capture every public
field, compare the full setup-binding map with the independent compiler,
and require CSS/map absence plus empty errors/warnings/macro artifacts.

The oracle is the repository's exact official Vue 3.6.0-rc.9 compiler/runtime
graph, not the report's uninstalled rc.10. Its public compileScript compiles
the unchanged original descriptor with explicit Vapor selection and inline
rendering. The reported TS module uses the existing locked Vite TS transform.
Babel parses complete actual modules and changes only real Vue imports for
loading; no emitted render is reconstructed. The existing mounted observer
compares all rendered text/attributes/form values while excluding backend
anchor comments; setup creates the original genuine reactive state.

Original interpolation/v-for failures reported upstream remain separately
unfinished and outside this finite qualification. Actual Vapor SSR still uses
the existing explicit standard-SSR fallback; this change provides no native
product, Vapor SSR, Chromium, all-fixture or performance acceptance.

TODO: exact source Actions including whole-module runtime, root peer review,
protected full suites and all 104 immutable instruction ceilings, signed
actual main with reporter trailer/issue closure, then a verified frequent cut.
