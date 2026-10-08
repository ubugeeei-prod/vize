# Source-derived complexity calculation guides

Paired issue: [#2748](https://github.com/ubugeeei-prod/vize/issues/2748).

## Decision

Expand the existing English and Japanese cross-file complexity guides from
the actual implementation at signed `6d26d84b4240e9356cf5078b6d277e181d311a42`.
Keep the three views distinct: own template decisions, distinct reachable
rendered template facts, and the seven weighted project dimensions.
No score, threshold, source pipeline, rule preset, diagnostic, fixture or oracle
changes in this documentation slice.

The guide traces region depth, conditional-expression depth, logical AST
boundaries and operator runs. It supplies every contribution to the existing
13 / 25 lint example, the small-parent 2 / 1 example, the existing Page/Grid/Card
rendered 7 / 4 fixture, mutual recursion, and the existing weighted unit input
whose seven dimension scores total 96. These are explanations of committed
controls, not newly executed product or corpus observations.

The precise coverage boundary is part of the explanation. Fallback selection
adds no decision or nesting, while its contents are visited at the existing
depth. `templateMaxNesting` measures nonempty template regions, not ternary
depth. Current handler-body carriers are not traversed by the expression pass;
original-only loop heads contribute structure without a collection AST.
Unknown expressions score zero and remain visible as unknown rather than
being interpreted from text. Update the existing metric specification to state
these boundaries without changing its historical distribution or pinned limits.

Use the native Vite+ integration first: `defineConfig` from
`@vizejs/vite-plugin/vite-plus`, `lint.vize.rules`, and `vp run lint`.
The generated all-rule reference and example leaves remain owned by their
separate documentation slice. Existing routes/navigation stay unchanged.

## Validation and remaining work

Check arithmetic, code-example parity between English and Japanese, source
references, Markdown structure, formatter output, and current-head Actions.
Do not rerun the historical corpus, build Rust locally, add mirrored tests,
or imply fresh runtime evidence from the source trace.

French, Brazilian Portuguese and Simplified Chinese guides retain their
existing algorithm detail. Translating the expanded explanation into those
three locales remains unfinished. Their existing enablement snippet is
updated to the same Vite+ integration so the configuration entry point stays
consistent across all five routes.

Statement-body complexity coverage remains unfinished as documented by the
current implementation; this slice does not add that product behavior.
The weighted score remains exploratory and is separate from the opt-in own
lint rule and Doctor's rendered-complexity notice.
