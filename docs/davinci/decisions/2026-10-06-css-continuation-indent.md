# CSS declaration continuation indentation (#7915)

The public 0.432.0 report supplies a complete 236-byte Button.vue: repeated
four-space formatting doubles value indentation, while tabs shrink it. The
verified reporter is ubugeeei (71201308). No related open PR or active owner
exists; older unpublished CSS experiments remain intact. This slice starts
from signed main c30f1a02da763e0086c489b8e17b9aa558187cff in a completed sparse
owned worktree, with no local build or dependency installation.

The existing printer rebases every leading byte as two-space structural
indentation, including authored whitespace in unparsed values. The authored
fallback also copies continuation indentation verbatim. For nondefault units,
reuse the existing CSS token iterator and rebase only complete declaration
whitespace gaps to one level deeper than the property. A colon candidate
commits only at a declaration terminator; an opening rule block rejects it.
Quoted strings and CSS escapes remain complete tokens, including interior
newlines. Generic printer structural indentation retains its existing scale;
authored fallback structural bytes stay authored. The default two-space path,
CSS parse/print pass count, errors, options, color/layout ownership and budgets
are unchanged. This is a legacy repair, not native migration.

The corpus retains the original issue and complete Button.vue, twelve whole
independently authored input/reference pairs (four spaces, tabs, tabs with
width four, default, CRLF, ordinary values, nesting, escaped string bytes and
selector colons), three public API passes and sixty CLI captures. CLI original
flags remain --no-config; explicit CRLF controls use the existing config path.
Every stdout/stderr/status/before/after value is retained before assertions;
stock Vue parse/style and DOM/SSR compiler vectors are retained in full. The
latter compiler modules must remain equal across indentation-only output.
No mounted browser/CSSOM or timing claim follows.

Prettier 3.8.3 whole original outputs and second-pass results for five option
sets are separately captured before executing the changed formatter. Its
property-only line layout differs from the report-permitted retained-first-value
layout. Its [documented units](https://prettier.io/docs/options#tab-width) guide
configured indentation; these references are never generated from Vize.

TODO: genuine exact-source Actions, all original history/fixture assertions,
protected full instruction/Rust suites, actual signed merge and root-owned
next-patch installed verification. Local Rust formatting and source syntax
checks are preparation only. The ordinary vp wrapper is unavailable in this
sparse tree; existing physical oxfmt/oxlint perform static checks without an
install. No old source-only review or stock execution grants current runtime
acceptance.

Initial source a594 Check37465495241 failed check-js solely because inserting
the owned canonical paragraph removed a required list-to-prose blank line.
Retain the whole failed log (599f3b70b9a7c3f5f3631cc637b5e0e2455d138f2573f06d4a38634e9d85030b),
restore the incoming blank and append the exact clause to existing prose.
All production/originals/references/protocols remain unchanged; fresh exact
source qualification is required without transferring prior runtime credit.

Current source 93dd24 Check37465930772 built successfully. Tooling shard1
retains two actual failures: the observational Glyph inventory lacks the new
test's existing L0 import row, and the new selector/custom-var reference wrongly
expects a retained newline inside var(). The unchanged printer legitimately
collapses that value. Independently authored single-line correction now equals
the whole stock Prettier 3.8.3 output and stock fixed point. Preserve the old
whole reference/corpus at the immutable a594/93dd Git commits and raw failure
log e693e58793be9e10767a6fa1105ab4a5588228eabbfce0ebee2157c3705f4b6d.
Only that new control reference changes; all twelve inputs, eleven other
references, original/legacy goldens, production and counters remain fixed. Add
only the exact source-witnessed generated L0 row. Eleven CLI subtests passed,
but complete new source runtime/Rust/protected qualification remains required.

The independent source reading qualifies the reported declaration/list values:
selector-colon candidates cannot survive an opening rule brace, and whole
quoted/escaped tokens remain opaque. Custom-property values containing curly
component blocks are outside this issue's proven depth+1 scope and retain
inherited raw-indentation limitations; no blanket custom-value repair is claimed.
Current bfa source passed the four Rust workers and all four tooling workers.
Actual signed main 97d5c26a153944e66439a3108746c19d7bf62220 now carries the
independent LSP option repair, so genuinely incorporate it with whole incoming
source/docs preserved and require fresh composed-source/protected qualification.
