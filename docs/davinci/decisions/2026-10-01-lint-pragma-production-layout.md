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

The fresh [canonical sweep](https://github.com/ubugeeei-prod/vize/actions/runs/36832691488)
at `229cd356d20eab8ad7989ffc0fa789d8a44e6693` passed its compiler corpus job:
production SSR compared 42,295 templates with zero errors, rejections or
divergences; the template emitter compared 42,276 with zero divergences and
19 existing legacy-error skips. DOM compared 42,279 with zero divergences and
16 existing error skips. Production SSR reached the native L4 plan for 42,229
templates; native Vapor reached 18,939 (44.7%) and retained its existing deferred
routes. Reach measurement is separate from the dedicated production Vapor
runtime parity suite and does not establish runtime parity for all templates.

The fresh [main baseline](https://github.com/ubugeeei-prod/vize/actions/runs/36833289934)
at `3924c20` reproduced 75 DOM and 18 production SSR divergences; the repair
sweep above reduced both to zero. Main and this repair have identical shard-0 core-tool, LSP and lint
failure summaries: non-normalized typechecker JSON paths, completion-count
drift and lint divergence/oracle-range failures. Typecheck divergence also
fails on both. Those cross-product failures remain release-promotion blockers.
Exact-head Actions and the full merge queue remain required for this repair.
Compiler fix-history closure and stability promotion remain separate gates.
