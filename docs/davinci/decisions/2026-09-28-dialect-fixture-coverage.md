# Dialect fixture coverage (#6892)

The pinned 146-project compatibility ledger now has a `dialectCoverage` row for
every gitlink. `unknown` means no dialect was established by the checked
evidence. `partial` names dialects observed in that project; it never says
every source file uses that dialect. A claim cites an exact pinned-registry
note or a repository test oracle, and validation rejects stale selectors.

The inventory has partial evidence in fifteen projects: eight Vue 2 SFC
projects, one Vue 2.7 SFC project, one Vue 3 SFC, two petite-vue corpora, two
Pug corpora and one Vapor JSX corpus. The other 131 remain unknown. The six
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

Remaining #6892 work: add exact case labels for the shared compiler, linter,
formatter, typechecker and LSP corpus; pin real Babel JSX, TS and TSX witnesses;
add grammar-generated Vue 0.x/1.x and quirks cases with pinned generator inputs
and reference behavior; then make #6854 aggregate each dialect and unknown
bucket. Keep these as unfinished until actual case execution and provenance
are recorded.
