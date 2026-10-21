# Dialect fixture coverage (#6892)

The pinned 149-project compatibility ledger now has a `dialectCoverage` row for
every gitlink. `unknown` means no dialect was established by the checked
evidence. `partial` names dialects observed in that project; it never says
every source file uses that dialect. A claim cites an exact pinned-registry
note or a repository test oracle, and validation rejects stale selectors.

The inventory has partial evidence in fifteen projects: eight Vue 2 SFC
projects, one Vue 2.7 SFC project, one Vue 3 SFC, two petite-vue corpora, two
Pug corpora and one Vapor JSX corpus. The other 134 remain unknown. The six
additional Vue 2/2.7 projects are backed by `package.json` Vue declarations
at the exact registered revision and Git blob SHA, alongside registered Vue
source globs. A package declaration proves its version selection, not that
every source uses version-specific syntax or that the project builds. The
offline validator binds each receipt to the registry revision and checks the
Vue 2 range; the pinned Git blob is the reviewable upstream evidence.
Vue 2.7 selected against a Vue 2 source in a test is **not** counted as a
pinned Vue 2.7 project. Babel JSX, Vue 0.x/1.x,
quirks and JS/TS/JSX/TSX are still zero at this project level unless a pinned
source witness or case selector establishes them. The report exposes zero
counts rather than turning a project name or file extension into a claim.

The #6891 differential case manifest has its own `dialectCoverage` metadata.
The project ledger's presence claims do not populate that field automatically:
one project may contain multiple dialects. The #6853 denominator retains
missing, skipped, errored and unknown cases, and #6854 cannot count a project
presence claim as a per-case zero-fallback pass.

The current compiler, formatter, linter, typechecker and LSP manifests now have
exact case labels wherever immutable authored source bytes establish a dialect.
The Babel compatibility corpus now pins `jsx-babel` to the actual
`@vue/babel-plugin-jsx@2.0.1` oracle. Its 100 authored inputs (98 JSX and two
TSX) join exactly to 100 recorded Babel adapter results, and the Rust oracle
executes the same rows through Vize. This remains provisional: the current
semantic verdicts are review-checked rather than runtime-equivalence results.
It grants no readiness credit and remains outside #6854's registered product
denominator until a product comparator admits these rows.

The two grammar-generated quirks inputs now join the compiler manifest by their
original inventory IDs, SHA-256 values, Vue 3/quirks selectors and parser mode.
The adapter wraps those exact template bytes in a deterministic SFC transport,
executes the source-built compiler with `--template-syntax quirks`, and compares
the complete SSR module bytes. These rows verify the `vue-quirks` dialect label,
while their native lane remains explicitly unsupported and receives no native
or equivalence credit.

Remaining #6892 work is to promote the five grammar-generated Vue 0.x/1.x
inputs into product manifests with actual adapter results, and make #6854
aggregate each dialect and unknown bucket. Keep these as unfinished until
actual case execution and provenance are recorded.

The first shared case-ledger slice makes coverage a common manifest contract.
Every planned case in compiler, formatter, linter, typechecker and LSP now
loads with an explicit coverage row; an omitted row normalizes to `unknown`
without changing the immutable manifest digest. Native-acceptance measurement
reports verified and provisional dialect rows separately while retaining every
nonverified case in the unknown readiness denominator. Verified labels are
restricted to the canonical #6854 vocabulary.

Fifteen of the seventeen ordinary compiler SSR cases have bounded
`source-inspected` evidence for the default Vue 3 SFC and HTML-template
selection; three typed-script cases also record `ts`. Their input bytes were
already pinned by SHA-256, and the compiler loader binds these labels to its
`--no-config` Vue 3 adapter. This is provisional measurement only. Together
with the two verified quirks rows, the compiler manifest has nineteen cases;
whole-product native compiler execution remains unsupported for all nineteen.

The typechecker registry also derives bounded `source-inspected` labels from
each immutable authored input after its SHA-256 check. Sixty of 61 cases
contain Vue 3 SFC and TypeScript witnesses, 59 include HTML templates, and three
include TSX. The remaining Nuxt dependency-identity case contains only its
pinned project graph, so it stays unknown instead of inferring a dialect from a
product name. The loader re-derives the labels from the pinned bytes and rejects
manifest drift. These provisional labels do not change the zero-fallback
denominator: native typechecker execution remains unsupported for all 61 cases.
Existing formatter labels contribute eighteen provisional cases among twenty
planned cases.

The linter loader now re-derives source-inspected labels for all 79 planned
cases from the five SHA-pinned authored cohorts or the separately pinned input
files. The LSP loader does the same for all 42 planned cases from each fixture's
immutable runtime files. The manifests cover Vue 2 and Vue 3 SFCs, HTML and Pug
templates, JS, TS, JSX and TSX where those bytes exist; generic JSX is not
promoted to the distinct Babel JSX dialect. Loader checks reject stale labels.
All 121 linter and LSP cases remain in the unknown readiness denominator because
source inspection is not native acceptance. The five Vue 0.x/1.x input-only
rows remain outside the executed denominator; the two quirks rows are now
registered compiler coordinates whose native result is still unsupported.
