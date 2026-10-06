# Hugged multiline interpolation indentation (#7969)

The complete public 0.432.0 report supplies a 299-byte Names.vue and its exact
formatter config. In a sole `<span>{{ ... }}</span>` hug, the existing template
formatter uses the child depth for the mustache, adding one unwanted indentation
unit to its expression and closing braces. The verified reporter is ubugeeei
(71201308). The source starts from signed actual main8fabdbcafd in an owned,
completed sparse worktree; no local native build or dependency installation.

Borrow the existing parsed ordinary opening tag range and existing chunk
adjacency. Only when the complete interpolation directly touches that opening
and its matching closing tag, use the parent depth for its syntax layout. The
same rule covers the existing first-pass JS wrapping path, preventing a later
indentation change. Keep block mustaches, prefix/suffix text, multiple mustaches,
nonadjacent closings and raw JavaScript literal bytes with their existing owners.
No additional pipeline stage, format pass, allocation or lexical scanner is
introduced. The wrapped-text path validates a sole mustache with the existing
range parser only after expression formatting reports a newline.

The regression corpus retains the whole original issue, input and config,
thirteen independently authored input/reference pairs, three API passes,
65 complete CLI captures and 52 whole stock Vue parse/DOM/SSR observations.
The original command uses discovered original config; check never writes,
three writes must equal the authored complete reference and the final check
must pass. Persist raw stdout/stderr/status before file reads, and complete
compiler observations before assertions. No mounted browser/runtime or native
route credit follows from these compiler vectors.

Separate existing Prettier 3.8.3 execution equals the whole original299B,
originalCRLF312B and corrected wrapped208B references, with full second-pass
equality. These stock references precede current Vize execution.

Exactly two old #6882 rows embody the same faulty hug:
prepared/sfc-layout-wrapped-interpolation and prepared/template-wrapped. Correct
only their current binary snapshots; retain the complete old goldens as corpus
assets, all original inputs/options/whole Rust laws and all300 manifests. The
current-reference adapter requires exact ID/API/kind/options/witness and complete
input/historical/current hashes. Historical output comparisons stay DIFFERENT;
separately counted current comparisons must be EQUAL. All unrelated references
and historical acceptance remain strict. Preserve the existing complete
suppression law while binding its actual current owner hash. Generated Glyph
inventory changes only the actual text import position and new law import row.

Paired decision: [issue comment](https://github.com/ubugeeei-prod/vize/issues/7969#issuecomment-6019346568).
TODO: genuine exact-source Actions, all original fixtures/history, protected
full104 instruction and full Rust suites, actual signed squash merge with a
parsed terminal reporter trailer, then root-owned installed next-patch replay.
Local source formatting, pure metadata tests and stock-only observations do
not grant current CLI/native/performance/publication acceptance.
