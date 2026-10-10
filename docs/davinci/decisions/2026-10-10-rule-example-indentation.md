# Preserve authored rule-example indentation

Issue: [#8363](https://github.com/ubugeeei-prod/vize/issues/8363).

## Decision

Rust documentation comments use `//!` with an optional single ASCII space as
their transport prefix. Remove that prefix and at most that one separator.
Every subsequent space or tab belongs to the authored example. Do not trim,
dedent, reformat, or repair intentional formatting violations in Bad examples.

The previous extractor called `trimStart()` on every line. The original
`vue/no-deprecated-slot-attribute` documentation has two-space and four-space
nesting; extraction flattened all of those lines. The component-options TypeScript
example also lost its object's indentation. The source files themselves were correct.

Keep every existing template/script/style/petite-vue wrapper, manual example,
override, rule implementation, explanation, diagnostic oracle and geometry gate.
Regenerate only the reference/catalogue code affected by the corrected extraction.
This is documentation-source fidelity, not a compiler, formatter or lint-policy change.

## Source and regression contracts

`docs/scripts/rules/indentation-examples.ts` authors four complete Bad/Good
references from the original Vue and TypeScript Rust documentation, original
petite-vue HTML documentation, and existing manual CSS carrier. Each reference
binds the whole source file's SHA256 and retains the complete expected example
objects, wrappers, language and evidence path. The references are independent of
generated Markdown and browser observations.

`tests/tooling/docs-rule-example-indentation.test.ts` runs the real `ruleExamples`
collector. It compares the whole original input/output packets and four additional
transport-boundary controls: spaces, tabs, uneven indentation, trailing spaces,
whitespace-only lines and a heading with no separator. These controls preserve
intentional Bad formatting through each Vue/TypeScript/HTML/CSS wrapper.

The unchanged original extractor fails six of these eight laws: the two original
nested examples and four transport controls. The original HTML and manual CSS
controls pass. The corrected extractor passes all eight. Retain the original
failure and source; a successful source test does not establish browser delivery.

The existing Docs renderer calls `verifyRenderedRuleIndentation` through
`verifyRenderedRulePackets` on English and Japanese rule catalogues. It uses a real
browser to fetch the actual generated reference HTML, checks all four original
code blocks and their Bad/Good ordering, and compares both complete example texts
with the independently authored original-source references. Generated command-tab
alternatives remain distinct from original authored fences. The receipt retains
the complete original source, expected/actual code and actual fetched HTML.

All existing catalogue, copy, native rule, geometry, theme and route controls remain
mandatory. This new source join supplements them; their expected text/counts and
budgets are not relaxed to make indentation pass.

## Composition and delivery

The separate #8334 inline-category work owns its existing source and geometry.
The indentation correction follows its genuine current parent as a third native
Stack child. Preserve the first two heads unless their owner performs an approved
coherent composition for a real source/review failure. Generate the child's final
catalogue from that actual parent so no older flattened copy is republished.

The original local reproduction and deterministic generator check are preparation.
Before delivery, require the final source's ordinary Actions, native rule example
qualification, full Docs renderer, genuine review and protected actual merge.
Keep #8363 open until representative nested Vue/TypeScript/HTML/CSS source and
rendered text qualify on the actual deployed site with whole artifact custody.
Do not infer publication or stability from a local test, old parent or queued PR.
