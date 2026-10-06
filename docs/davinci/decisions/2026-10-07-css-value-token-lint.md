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
