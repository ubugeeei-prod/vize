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
Preserve zero or one authored blank line before a sibling block-rule prelude
when the preceding brace already occupies its own authored line. Compact source
blocks retain the historical printer gaps before single/at-rule preludes. A
complete actual multi-selector prelude also owns its preceding gap when its
compact selector list is corrected; the immutable Unicode/color-offset control
exercises this precise boundary. A complete owned nested style prelude
also guards the gap after an authored declaration terminator.
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

Current status: independent Draft PR #8087 has genuine source Actions failures
retained below. Static source review and the three independent original references
pass; they grant no current execution credit. Resolve actual failures before
Ready/auto/queue admission. Protected instructions, actual merge and installed
release replay remain TODO. PR #8076 delivery has priority when the security fix
actually merges.

Source peer found that `fmt` always prints its unchanged progress/summary to
stderr. Each fixture now pins all five complete stdout/stderr/status outcomes
from the unchanged public CLI producer; no messages are normalized. Raw process
results are persisted before reading output files, and read failures are retained
without losing streams. This observer repair grants no formatter execution credit.
Compiler observations are also persisted in full before unchanged diagnostic
assertions, so an unexpected parse/style/DOM/SSR vector remains inspectable.

The first real source Check 37411815094 failed: nested-rule leading whitespace,
legacy compact-block and vendor-keyframe whole outputs, one stale generated line
number, and missing Chromium in the actually selected tooling shard. The current
correction preserves every original source/expected file and old snapshot, retains
compact-source normalization, and covers the declaration-to-nested-rule gap only
with the same full parse-prelude/token authority. Regenerate genuine inventory
positions. Provision the tests-package Chromium for the exact selected browser
test using the existing complete tier/shard selector and stable apt path, and
for full unsharded Check. No test is skipped, no ceiling rises, and no old
snapshot is updated. New source execution and protected acceptance remain TODO.

Source c00 Check 37413939455 reports two genuine no-floating-promises
warnings in the new pure browser dependency test. Await both actual Node test
promises instead of waiving the zero-warning gate. Their assertions and every
production/oracle byte remain unchanged; preserve the failed raw check-js log.
Healthy source formatter/Rust/tooling/browser jobs continue before the next
necessary source push. This correction grants no runtime or merge acceptance.

The actual c00 tooling worker rejects the expanded PR workflow under three
unchanged 350-line laws. Keep those exact laws and compact the dependency hook
into the existing preparation action: retain the original two Pkl/lint commands
and inherited plan/tier/shard environment, then select the tests-owned browser
with the same validated selector. The full unsharded caller still installs it.
The workflow returns to 350 lines; no native guard, selector or ceiling changes.
The existing source-selection contract still compares the exact original two
commands inside their action. Its local replay lacks the yaml package and grants
no acceptance; the other eleven pure checks pass, including all three original
workflow-cap/guard laws. Genuine hosted execution remains required.

The complete c00 tooling artifact authenticates all 17 original/expected carrier
bytes. All three original cases and the CRLF original qualify with 20 complete
CLI captures. The 13 template controls stop before CLI because this diagnostic
helper loaded Vue runtime before the unchanged shared HappyDOM initializer. Read
the same pinned Vue package manifest for version identity instead; let the
existing runtime helper install document before loading Vue. Keep all actual
DOM/SSR diagnostics and runtime assertions; no consumer is mocked or skipped.

The c00 Rust failure is exactly the existing keyframe-comma control: the printer
adds one empty line between two frame blocks. Its complete expected bytes remain
unchanged. Primary Lightning CSS keyframe nodes have no locations. The already
parsed Keyframes parent may own only its direct token-depth frame headers, after
the complete observed frame count equals its parsed list. Each retained whole
header still joins its brace ordinal and full original/printed token equality.
Frame commas remain non-selector punctuation; compact historical frame gaps stay
under the existing compact-source policy. Add no parse, printer or format pass.
The existing compact/vendor-keyframe Rust shard passed at c00; this is distinct
from the failing new case and does not grant full-history or instruction credit.

The full Check workflow also grew from 690 to 692 lines and was genuinely
rejected. Remove only two empty YAML separators, conserving its whole parsed
workflow after removing the new browser caller; it returns to 690. No old step,
command, environment, gate or ceiling is removed. PR workflow is 350 lines.
The corrected successor still requires fresh source Actions, all 17 API cases,
all 85 complete CLI captures, history and protected gates before admission.
