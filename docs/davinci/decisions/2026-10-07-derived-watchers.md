# Derived watcher advice and required markers

Issues: #7899 and #7901.

`script/prefer-computed` previously treated any `.value` assignment anywhere in a
watcher as computed-compatible. The replacement proof requires one synchronous,
unconditional expression statement assigning a pure expression to a declared
Vue `ref` or `shallowRef`. The expression must depend on the callback's first
parameter or the watched source and must not read the target's prior value.
Calls, async/await, conditional statements, old-value/cleanup parameters,
side effects and other writes suppress advice.

The shared script Program supplies declarations, import aliases and mutation
evidence. Top-level ref declarations are eligible; same-named bindings anywhere
make resolution ambiguous and suppress advice. Vue imports and unbound
auto-import names are supported; unrelated locally bound factories are excluded.
The already shared template AST supplies `v-model`, inline assignments/updates
and template-ref evidence. HTML comments, ordinary text, static attributes and
`v-pre` do not manufacture writes. Ambiguous template expressions can only
suppress advice; token over-collection never creates a finding. Components with
a sibling script block or an unobservable template suppress advice because
absence of other writes cannot be established.

`a11y/use-list` now requires non-whitespace list-item content after its existing
bullet prefix. A standalone `*`, dash, plus or Unicode marker is clean in both
Vue and JSX. Real bullet text and semantic list-context boundaries are retained.

The corpus under
`tests/_fixtures/differential/linter/derived-watchers-7901/` preserves the full
public SFC reproductions and inverses for editable state, DOM refs, template
read-only uses and inert comments/string literals/`v-pre`. Unit coverage
additionally exercises callback purity, source dependency, aliases, shadowing
and external writes. Hosted exact-head Rust source checks run this corpus.
This change records legacy regression evidence only; no native parity, pipeline
replacement or performance improvement is claimed.

Validation status is reported on the owning issues and PR; it is unfinished
until hosted Actions completes. Root orchestration owns merge-queue admission
and actual merge verification. n8n upstream is read-only.
