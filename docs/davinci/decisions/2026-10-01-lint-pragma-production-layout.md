# Lint pragmas and production layout

For [#6100](https://github.com/ubugeeei-prod/vize/issues/6100), the canonical
real-project sweep reproduced 18 SSR production divergences on 42,295
templates. External lint pragmas remained in the compatibility parser when
comments were disabled. Codegen omitted their text, but branch-fragment and
whitespace decisions had already counted them. Native lowering dropped them.

Keep external eslint/oxlint pragmas on comment-aware tool surfaces only.
Comments-off compiles discard them before whitespace and structural transforms,
as the official compiler does. Vize directives retain their existing policy.
This preserves lint suppression metadata when comments are requested.

The owning SSR fixture is registered in both the native/compatibility corpus
battery and the production Vue render oracle. Exact HTML assertions cover
conditional raw HTML, fragment boundaries and loop-interpolation whitespace.
The production Vapor reactive/event fixture also carries a lint pragma.
Parser coverage verifies all eight pragma forms in both comment modes.

Acceptance requires exact-head Actions, the full merge queue, and a fresh
canonical real-project sweep. Compiler fix-history closure and release
promotion remain separate gates.
