# CSS selector lists and adjacent rule layout (#7926, #7966)

The original reports contain three complete SFC inputs. Their expected outputs
are unchanged. The inputs and issue bodies remain byte-pinned in
`tests/_fixtures/differential/formatter-regressions/css-rule-layout-7926-7966`.
Existing installed Oxfmt 0.63.0 and Prettier 3.9.6 independently produce exactly
those three outputs. This does not claim the report's Oxfmt 0.64.0 was executed.

Use the first existing Lightning CSS parse to identify block-rule preludes and
actual multi-selector style rules. Keep owned prelude ranges and opening-brace
ordinals, rather than reusing byte offsets moved by existing color protection.
A prelude must equal the original whole bytes at its corresponding brace. Only
an entirely matching original/printed token stream may commit whitespace edits.
Print each actual top-level selector separator on a new rule-indented line.
Preserve zero or one authored blank line before a sibling block-rule prelude.
Declaration values, strings, escapes, functions, attributes, custom-property
blocks and keyframe selector commas do not acquire style-selector ownership.
The existing authored-syntax fallback, comment policy, options, selective fixed
point passes and errors remain the producers. Add no parse or formatting pass.

The 17 independently authored whole cases include all three originals and 14
controls. The existing numeric-spelling and keyframe layout contracts remain
controls; the reference tools differ there, so no general CSS parity is claimed.
CRLF uses Vize Auto; Oxfmt 0.63.0 rejects Auto and requires an explicit CRLF
reference option. The source test will require three whole API fixed points.
The ordinary hosted CLI test will retain 85 complete processes, all whole output
files, original and formatted parse/style/DOM/SSR diagnostics, complete Chromium
CSSOM/computed-style observations and complete Vue DOM/SSR control observations.
The two style-only originals retain their complete missing-template/script
parse diagnostics; no component is invented to erase those negatives.
Route handling remains uncredited (`nativeHandled: 0`) without measured evidence.

Current status: the publication hold is lifted and this slice is being sent to
an independent conventional Draft PR and ordinary source Actions. Static syntax,
rustfmt and the three original independent references pass. Current formatter
execution, history, full Rust and instruction gates are still unknown. Bounded
source review runs alongside Actions; genuine failures must be resolved before
Ready/auto/queue admission. Actual merge and installed-release replay remain
TODO. PR #8076 delivery has priority when the security fix actually merges.
