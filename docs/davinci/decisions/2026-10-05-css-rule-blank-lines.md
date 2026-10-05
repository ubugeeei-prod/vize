# Preserve authored CSS declaration groups (#7826)

Issue: [#7826](https://github.com/ubugeeei-prod/vize/issues/7826).
Paired [source decision](https://github.com/ubugeeei-prod/vize/issues/7826#issuecomment-5987109062).

Actual main `d1a25ec1da2efca98534520ff8ebe84d34708e3d` retains the existing
LightningCSS parse/print, color protection, selective fixed-point stabilization
and authored-token fallback. The original Flat.vue loses a separator in normal
printing; Nested.vue's animation and `.5` values select the layout fallback,
which also drops separators. This is a source explanation of the reported
regression; fresh execution must establish the repaired output.

A private pass after the existing print/selection restores at most one authored
blank line between declaration or child-rule groups inside a rule. It requires
matching complete significant token streams before publishing any replacement,
keeps quoted strings, CSS escapes and complete comments indivisible, ignores structural punctuation
inside parentheses/brackets, and copies the printer's indentation and selected
newline. A token mismatch returns the existing formatted output atomically.
No new parse, pipeline stage, serialization or CSS bypass is added. Existing
nested/top-level comment and SCSS policies remain intact. In-rule comments still
select the original raw-content path before this helper; two whole SFC/CLI
controls freeze comment punctuation and the real grouping around it. Already
matching output returns before the full token walk, including the ordinary
protected CSS fixture with its between-rule separator.

The additive canonical JSON corpus pins 27 complete source/output/option plans,
including the exact three original issue inputs, scoped/module/opacity, collapsed
separators, after-child and media nesting, quoted punctuation/escaped quotes,
unquoted URL comment bytes, escaped Unicode/hex selectors, real comments,
CRLF/Auto, tabs/width and SCSS. Compact CSS must still be formatted, and a blank
line before a closing brace must not create a declaration group. Expectations
are authored references, not described as captured results. The public Rust
APIs compare every complete output across three passes. The existing authentic
formatter observer build executes the same plans in its existing merge-only
full suite; ordinary source Rust checks prove the API laws first. Nine default SFC plans also
use the existing source-built CLI and exact check/dry/write/recheck status,
stdout/stderr and whole file bytes. A bounded always-retained report binds the
corpus, actual API/CLI build receipts and raw process observations, including
partial failures. No extra observer build or CI stage is introduced. Four
standalone style controls retain the existing no-final-newline API contract
for reindentation or comment-aware chunks; this repair does not alter it.

Original differential packs, defaults and all 104 instruction ceilings remain
unchanged. The protected SFC input includes ordinary CSS and its separate
between-rule blank line: actual unchanged-budget acceptance must prove this
repair has no regression; do not infer acceptance from source checks.

Independent source review clears complete print/token alignment, escape safety
and structural-gap ownership. The direct comment law is a defensive lexer
contract; it adds no claim that existing nested-comment contents are formatted.
Current state: source review and formatting only. Fresh exact-source Actions,
protected complete suites and unchanged caps, then actual signed merge are
required. Remove a known-red candidate and fix its source; do not waive a gate.
The verified reporter trailer is
`Co-authored-by: ubugeeei <71201308+ubugeeei@users.noreply.github.com>`.
This independent wt branch starts from actual main; no native Stack dependency
or native/history/default-replacement credit is claimed.

TODO: post-source/protected terminal receipts and actual signed merge here and
on the issue. Publication/public installed-CLI proof follows the release owner's
first-cut fence and explicit thaw. Broader style/dialect/native history remains
unfinished under the existing roadmap.
