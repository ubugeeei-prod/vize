# CSS lint declaration ownership for #7989

The [public report](https://github.com/ubugeeei-prod/vize/issues/7989) contains
four enabled rules. Its original expected result is clean, while the reported
0.432.0 output contains three script and three CSS warnings. This slice fixes
only the CSS rules; the separate script slice is #8135. The paired source
[decision](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6020155298) records the same scope and delivery plan.

`css/no-v-bind-performance` formerly searched all raw style bytes for
`v-bind(`. `css/no-important` searched raw bytes and skipped comments, but
still treated a quoted `!important` as a declaration flag. The unchanged
397-byte original contains both patterns in an old-rule comment and in a
quoted `content` value. The fixture retains the full Vue input, 235-byte
four-rule configuration, original argv, 3123-byte issue and 725-byte reported
output, each with its independently verified public-source SHA-256.

Both rules now use the existing cssparser tokenizer and declaration grammar
inside their existing checks, after the unchanged Lightning CSS acceptance
gate. Declaration values provide original byte spans. Comments, quoted
strings and URL tokens remain opaque; selectors and at-rule preludes do not
provide declaration values. Nested rules, declaration at-rules and custom
values retain real function tokens. A real `v-bind` token must also have the
literal authored `v-bind(` spelling; a different or escaped CSS function name
must not acquire Vue runtime-binding semantics. A terminal important flag
retains its entire authored spelling, including allowed trivia and case.
Tentative declaration findings publish only after the declaration succeeds,
so retrying a nested selector cannot leak a function finding.

The public CSS rule interface and existing pipeline stages are unchanged.
Each enabled rule obtains its own token span traversal of the already
accepted style; no compiler stage, provider query, serialized packet, source
mask, generated replacement text or new dependency is introduced. Diagnostic
message/help/severity and old genuine-call unit bodies remain exact. Existing
style offsets and disable-comment handling still own the public ranges.
Instruction budgets remain unchanged and require genuine protected proof.
Absent `!` or literal `v-bind(` candidates, the corresponding existing check
returns without a token traversal; these probes never classify or alter CSS.

Seventeen independently authored complete service/CLI vectors cover the
whole original LF/CRLF/Unicode input, escaped/quoted/comment/URL text, genuine
and nested calls, quoted function arguments, important trivia/case, nested
rules, font/keyframe declarations, selector/at-rule preludes, nested custom
values, both suppression modes, multiple style blocks, empty and invalid
stylesheets. Five token laws preserve complete authored span arrays. The
service test retains all cases before its terminal assertion. The CLI test
retains all eighteen invocations' full raw stdout/stderr/status/argv under a
unique persistent directory in the existing uploaded nextest artifact root,
checks all original physical pins, and compares entire public reports.
These are authored expectations, not recaptured output or partial matching.
The report keeps its existing character-column contract; service ranges
remain physical UTF-8 byte offsets, including CRLF and non-ASCII prefixes.

Preparation on actual signed `064ffb3b1adea91837ed9bbdeeeef9a363002efd`
retains all three pending script findings in the complete original vector.
Configured rustfmt and both existing source inventory checks pass locally
(20 Croquis files, 19 migration files); only the two actual Patina inventory
rows change. Product/API/CLI/token runtime execution remains unprovided.

The genuine native Stack will use the meaningful published #8135 script
successor as parent. The CSS child preserves all parent source/config/report
pins and real CSS positives and changes only the complete source-proven
original CSS findings under the combined source. A qualified parent prefix
can deliver independently. The shared shell-quote repair is owned by #8136;
security-red source remains Draft/offqueue until actual security-fixed main
and fresh qualification. No native/history gate is promoted by this fix.
The entire #7989 remains open until both slices and the full original are
actually accepted; public installed release verification belongs to root.

The combined source now genuinely descends from published script parent
`a77c035c8323051b7a1a8ae945f05cd0cc336f09`, itself on actual signed security
merge `ef84821d30fa0d8538b2472fef34418e75380523`. The
[paired composition](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6033182729)
records the authored full original clean result under both corrections.
Only the first three complete parent vectors change from three CSS warnings
to zero; all other 38 parent cases, including real CSS positives, remain
whole-object exact. The same three own vectors change from pending script
warnings to zero, retaining all other 14 controls. Original input, config,
command, report and issue pins remain exact. The original plain expectation
now requires the entire unchanged clean-report boundary, without filtering.

Independent source review of `83b0b65448b13d3a6c39be3f9dabbebc185ed6fb`
clears the declaration/token authority and staged fallback. All production
and token-law blobs retain those reviewed bytes after the genuine parent
replay. This review supplies no compile, runtime or instruction acceptance.
Configured rustfmt and existing 20/19 source inventory checks pass; fresh
source Actions, all seventeen service/eighteen CLI observations, parent
whole 41 vectors and protected unchanged 104 budgets remain required.

The first actual parent Actions audit on `a77c035c` found a new high
GHSA-6qxp-vccf-f47h in @modelcontextprotocol/sdk 1.30.0. Both layers remain
Draft/offqueue; the security owner supplies the genuine updated main before
fresh qualification. The earlier shell-quote fix is actual, but its green
result does not accept this new advisory or the changed CSS source. Native
Stack membership and protected actual delivery remain pending. #7989 stays
open; no installed, native, history or performance completion is claimed.

The [first hosted correction](https://github.com/ubugeeei-prod/vize/issues/7989#issuecomment-6033486602)
retains failed `d5f3659549`/Check37588634096. Tooling rejects one new CLI
legacy-storage import and css.rs606 versus its original605-line ratchet.
The CLI now uses identical L0 reexports. A move-only commit relocates the
entire old disable_tests module and its identical snapshot; the formatted
whole inverse restores every old byte/name. The production file shrinks to
424 lines and the moved test file is177, without increasing a limit.
The new CLI physical pin guard also used one excess parent directory:
CARGO_MANIFEST_DIR at crates/vize reaches workspace tests via ../../;
compile-time includes retain their correct independent ../../../ relation.
Only the physical constant changes; every pin and full comparison remains.

Authenticated d5 artifacts prove all17 complete CSS service comparisons,
all42 parent API and all42 parent CLI observations, including the whole
original zero report. Four official ZIP size/digest/CRC checks and literal
synthetic20c859/source-d5 whole-tree equality bind that evidence. The own18
CLI observations remain unexecuted because their guard failed first.
These scoped observations do not accept the failed aggregate, the current
successor, protected instruction gates or installed distribution. Current
SDK remediation, fresh full source proof and actual Stack delivery remain
required, with every original/control oracle and budget unchanged.
