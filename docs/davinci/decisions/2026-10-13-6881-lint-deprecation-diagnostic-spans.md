# Lint deprecation diagnostic targets

Date: 2026-10-13. Related issue: [#6881](https://github.com/ubugeeei-prod/vize/issues/6881).

The original v0.448.0 Matrix run [38331707741](https://github.com/ubugeeei-prod/vize/actions/runs/38331707741)
reports paired findings for three implemented migration rules. The native and
eslint-plugin-vue 10.9.2 severities agree, but their ranges identify different
pieces of the same deprecated syntax. The first six original shards contain
136 slot-scope pairs, 54 `.sync` pairs and six functional-template pairs.

Report `.sync` on the complete binding directive, `slot-scope` on the attribute
name, and a functional SFC template on its complete `functional` attribute.
These are the actual upstream diagnostic nodes. Recover the functional attribute
only within the descriptor's already parsed opening tag, respecting quoted
values; do not add a parsing stage. Keep the existing rule detection, messages,
help, severity, fixes and dialect gates.

The corpus retains the two complete Vue Element Admin files at revision
`6858a9ad67483025f6a9432a926beb9327037be3`, their original Matrix observations,
and twenty-four native API cases. Independent pinned-provider execution covers
short and long bindings, dynamic arguments, multiple modifiers, multiline and
Unicode values, boolean and valued attributes, misleading quoted text,
duplicate attributes, healthy replacements and neighbouring names. The native
corpus compares complete result packets, including messages, help, labels and
fixes. A petite-vue document remains a foreign-dialect control.

The original failed Matrix observations remain failed evidence. No comparator,
preset, rule map, documented divergence or zero ceiling changes. Actions must
qualify the new native implementation; a new Matrix is owned by the release
coordinator after actual merge. Other inheritance, props, slot, component and
baseline-parser failures remain separate work.

The initial source Actions passed the new complete 18-case native corpus but
refused old preset and LSP vectors. Their complete packets remain intact; only
the three affected rule ranges change, including UTF-8 bytes and LSP UTF-16
positions. The original failed run 38336568495 remains part of the evidence.

Historical preset corpora stay at their original paths and exact bytes. Current
native and CLI regressions read separate `current-range-cases.json` copies;
their complete authored vectors differ only at the approved diagnostic ranges.
Released typed-default and runtime-prop reference authorities are not repinned.

Review controls add six whole header cases and direct native rule/severity/UTF-16
range comparisons with the pinned reference. Empty `=` tokens are skipped; HTML
whitespace keeps newline-spanning attribute values intact. Matching only the
block parser’s spaces/tabs would diverge from the verified provider on those
newlines, so no opening-tag fallback or parser-syntax weakening is introduced.

The fresh main composition retains all 323 closed source rows and both historical
frames. A sealed whole-body receipt reconstructs and reverses the three approved
current-body edits; the other 320 rows and every non-current manifest field stay
byte-identical to main. Mutation, omission, reorder, extra-row, mode and source
refusals remain required. This source evidence grants no native or release credit.

The cumulative Stack composition keeps the original reference reader byte-exact
after Actions refused its changed identity. A separate current CLI reader proves
the complete range inverse; Maestro migration regressions use the matching current
fixture. Both prior receipts remain historical source-only evidence. The new
sealed 323-row parent composition changes three owned rows, preserves all 320 other
rows and both frames, and reverses only the coupled helper/law checksums. Later
physical sources are qualified by the unchanged original refusal family.
