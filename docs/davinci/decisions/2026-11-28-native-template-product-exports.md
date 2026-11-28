# Native template product exports

`VirtualCodeGenerator::native_template_operand_products` reads the selected
original L1 setup block and the admitted L2 template operands. Canon's expression
document and Maestro's template inputs still observe the same operands in authored
order. Setup export names come from those operands' original lexical references,
filtered to value declarations in the actual setup unit's root scope and sorted
as before. Selecting exports does not parse the script or expressions again.

The existing descriptor adapter retains its source, template and setup selection
checks before delegating. The shared script writer consumes the original setup
`SourceBlock`, preserving its source map base and complete output bytes. A late
reference projection refusal discards all products.

This remains an explicitly requested editor product. It does not switch the LSP
backend or establish complete native type checking. Unsupported input admission
stays with the original File producer. The production generators, product
dependency debt, zero-fallback qualification and stability period remain open
under #13020 and the completion gates.

## Qualification

The `template-tool-projections` Rust feature row runs both entry points against
the separate checker/editor oracle. It compares complete checker documents,
authored links, ordered expression inputs, script text, virtual URIs and mappings.
Unicode, entity decoding, CRLF, static and script-only inputs and descriptor
revision/selection refusals remain covered.

The `Native template products` workflow also runs
`original_tool_template_projections_match` on every tracked
`.vue` source, reporting refusals separately. Set `VIZE_TEMPLATE_PRODUCT_INPUTS`
to a NUL-separated inventory to select a hydrated external corpus instead. The
tracked repository scan does not claim full real-project qualification.

Its optimized, serialized `paired_template_tool_products` run compares complete
products at 1, 8, 32 and 128 repeated elements against separate checker/editor
generation. It retains paired time samples, exact allocation counts and peak
allocation bytes; a repeated slowdown requires a second longer reversed-order
run before failing. Exact-head Actions results must be recorded in the PR before
this slice is accepted.
