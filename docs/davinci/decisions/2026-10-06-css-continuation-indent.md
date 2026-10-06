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
