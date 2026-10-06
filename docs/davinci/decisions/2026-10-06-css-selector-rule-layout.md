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

Fresh 6f9 Check 37415785884 passes JS lint and Rust fmt but its Rust producer
112114089752 fails the new nested guard's Clippy collapsible_if rule. Retain
the complete actual log and use the prescribed let-chain with the same option
binding, observation order and body. No lint allow, ownership relaxation or
expected-byte change is introduced. Healthy tooling/DOM executions continue
before this necessary source successor; full Rust remains unqualified.

Security PR #8080 actually merged at signed-valid
`48cb1d4f35ebd0aa3824817c67752b5e4d0a9dbb`. Normally incorporate this
literal main before fresh source qualification, retaining its selected-PR and
queue strict UI policy and all owned production/oracle bytes. The preceding
6f9 tooling artifact independently qualifies all 17 cases/85 full CLI captures,
68 complete Chromium CSSOM/computed/HTML observations and 13 full Vue DOM/SSR
state pairs; Rust remains unqualified because of the retained Clippy failure.
Those positives do not transfer to the incorporated successor.

A concrete prospective merge with independent PR #8076 conflicts only at the
shared appended canonical line and attribute-file EOF. Relocate this slice's
complete owned canonical tail to the existing prose line 279, keeping all 350
incoming lines/prefixes. Place its exact formatter-only attribute block beside
the existing formatter block; removing it recovers the entire incoming file.
No attribute value/order for overlapping patterns, original byte, expectation,
production path, workflow policy or ceiling changes. Recheck actual prospective
composition before admission, then require fresh source/protected qualification.

The private first metadata placement at adjacent line 295 still conflicts with
#8076's line 294 under the actual Git merge algorithm. Keep that failed virtual
composition distinct; use the unchanged complete owned tail at distant existing
prose line 279, then require a clean actual prospective merge tree. No queue
admission or source-runtime acceptance was inferred from the failed attempt.

## Necessary canonical formatting correction

Current source42 fails only the JavaScript formatting check because placing the
complete clause on blank line280 immediately before a list requires another
separator. Append that exact clause to existing prose line279 and restore the
original blank line280 instead. All350 incoming line prefixes, reviewed source,
whole inputs and expected vectors remain; fresh successor checks are required.
The current42 formatter/Rust observations remain scoped to that source.

## Actual admitted-prefix composition

Current b555 passes all four required source checks and its complete17-case
API/CLI/browser/runtime corpus. Actual signed main8006 composes cleanly, but
the healthy admitted prefix through candidatec4 has its own clause on the same
canonical prose line279. Move only this complete clause to existing prose326
and normally incorporate actual signed8006, conserving all350 incoming line
prefixes, all reviewed product/corpus bytes, and every original expectation.
No unmerged prefix production is adopted. Require a clean actual-prefix merge
tree and fresh successor Actions before independent queue admission; previous
current checks remain historical evidence, not successor gate credit.

## Protected layout metadata cost

The paired [#7926](https://github.com/ubugeeei-prod/vize/issues/7926#issuecomment-6010944133)
and [#7966](https://github.com/ubugeeei-prod/vize/issues/7966#issuecomment-6010944334)
decision retains genuine candidateb424 Check37424004555 failure. All three
repeats measured SFC305832>293575 and reused-allocatorSFC287204>274723.
Only this PR was removed from the queue and returned to Draft. The current
complete Callgrind graph attributes18700 inclusive instructions to first-parse
layout collection, including12705 in the token iterator. Exact predecessor3f765
qualified at286685/267834. No binary-address or disassembly claim is made.

Collect optional metadata after the same existing print/reindent. If the entire
protected printed CSS, trimmed only at its document boundary, equals the entire
protected input and no raw comma exists, no selector or rule-gap edit is needed.
Keep an empty first-parse marker so later stabilization passes cannot acquire
replacement ownership. A comma or any printed change retains the original first
parse, whole-prelude and complete-token guards. Add no parse, print or pass.
All original/expected carriers,17 cases/85 complete CLI captures, runtime states,
options/errors, probes, allocator/window protocol and budget files remain exact.
Fresh source, protected104x3, actual signed merge and installed release replay
are still required; the source change grants no new performance or native credit.

## Script settings conversion

Actual candidate37095efe Check37428999341 failed all three original windows:
script929056>929044. Preserve the complete312 raw rows and priorf6b result;
remove only8087, keeping other deliveries active. The f6b metadata correction
measures SFC287333/reuse268484 under unchanged ceilings but grants no script
or final delivery acceptance.

The paired [7926 decision](https://github.com/ubugeeei-prod/vize/issues/7926#issuecomment-6011682120)
and [7966 decision](https://github.com/ubugeeei-prod/vize/issues/7966#issuecomment-6011682377)
remove repeated base-settings conversion from the existing script stabilization
loop. Retain one owned JsFormatOptions cache for that invocation only, initialized
after a successful parse; clone those same settings for each existing print.
Configured import sorting still clones separately per invocation, avoiding an
extra persistent configured-sort copy. Every parse has its original fresh arena;
all parser/printer calls, six-pass limit, skip/no-op/error/Auto behavior and whole
outputs remain. Remove only the unused private allocator argument, keeping
public signatures. The ten complete original/current script laws, all original
corpus and budgets remain byte-exact; update only actual source-owner hashes and
observed migration inventory. Fresh source/protected execution must establish
net cost because cloning may offset conversion savings. No native, speed or
installed-release credit is inferred.
