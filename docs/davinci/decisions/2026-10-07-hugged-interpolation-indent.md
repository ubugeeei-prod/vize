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

Before publication, genuinely incorporate signed actual main064ffb3b1a, which
contains the independent Canon readiness repair. The three reviewed production
blobs, complete originals/references, two historical refinements and counters
are conserved. Whole incoming source/docs survive the clean normal merge.
CRLF corpus carriers explicitly retain their bytes and declare cr-at-eol for
Git whitespace inspection; no output comparison or formatter rule changes.
Fresh exact composed-source Actions and protected qualification remain required.

[Paired control clarification](https://github.com/ubugeeei-prod/vize/issues/7969#issuecomment-6019573786).
Pre-execution source inspection corrected one newly authored control: ordinary
prefix/suffix text is handled by the unchanged inline interpolation printer,
which collapses its short expression. Its whole expected reference and first
check/write streams now express that existing behavior. The exact input,
production, two same-bug historical references and every other new reference
are unchanged. A fourth separate Prettier observation equals the complete
corrected prefix/suffix reference and fixed point. A separately retained width30
SFC stock observation chooses a block layout for p, so it is an alternative
layout, not an equality oracle; the public template history still qualifies
its retained hugged layout by the parent-anchor rule.

[Paired observed qualification repair](https://github.com/ubugeeei-prod/vize/issues/7969#issuecomment-6020026087).
Exact source585 Check37487541685 genuinely failed tooling1 after11 of13
cases (59 actual CLI calls/48 compiler observations) and one existing Rust
CRLF law. Preserve both raw failures and the complete old law. The two
first-wrapped cases passed their complete CLI references; their stock generated
modules differ because JavaScript expression newlines remain in the emitted
code. Independently captured original and already-authored reference templates
now provide separate complete Vue3.5.35 module oracles for only those two IDs.
All other eleven still require whole original compiler-module equality.
Two independent states per affected case additionally require actual mounted
happy-dom DOM and stock SSR HTML, including Unicode/escaping; retain complete
modules/runtime observations before assertions. The stock-only module capture
authenticates62 actual loaded files within the unchanged73-file physical
authority; it executes neither Vize nor these runtime controls. Fresh Actions
remain necessary for65 CLI/52 compiler/16 paired runtime observations.

The existing CRLF literal law exercises this same parent anchor. Correct only
its one complete expected syntax string from6/4 spaces to4/2; the entire
original source, `first\r\nsecond` literal content, all three fixed-point
assertions and every other law remain unchanged. Historical585 retains the
whole faulty expectation. No production/corpus input/reference/options or
legacy300 manifest changes accompany these qualification repairs.
Security is independently owned in PR8137; keep8134 Draft/offqueue until its
actual signed main is incorporated and fresh source/native/Rust/history and
protected instruction gates qualify. No failed-head acceptance transfers.
