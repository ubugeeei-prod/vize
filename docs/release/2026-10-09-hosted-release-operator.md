# Hosted official release operator

Tracking issue: [#8335](https://github.com/ubugeeei-prod/vize/issues/8335).

> **Current behavior (2026-10-17):** the operator uses Vite+, Node, Rust 1.99.1
> and rust-script, then installs the frozen JavaScript dependency graph. MoonBit
> tooling and registry preparation have been removed. The exact frozen H must
> pass full `Check` qualification (seven full-source jobs plus twelve crate semver
> checks) before Release starts. Fuzz, Miri, Real Project Matrix and Docs build
> run on their own schedules or manual dispatches. The operator checks the source
> PR's required statuses before requesting the metadata PR's squash merge through
> the GitHub API. GitHub applies the repository's existing merge rules; the runner
> verifies actual signed delivery and catalog identity. It has no independent
> protected full-workflow release wait.
>
> Current authorities: [operator workflow](https://github.com/ubugeeei-prod/vize/blob/55f9b9b8706f780a6fe60e2d0144cdf2248fc089/.github/workflows/release-operator.yml),
> [source qualification](https://github.com/ubugeeei-prod/vize/blob/55f9b9b8706f780a6fe60e2d0144cdf2248fc089/tools/support/release/pr_pin_runs.rs),
> [required Check jobs](https://github.com/ubugeeei-prod/vize/blob/55f9b9b8706f780a6fe60e2d0144cdf2248fc089/tools/support/compat/github/release-preflight-evidence.mjs),
> [metadata admission](https://github.com/ubugeeei-prod/vize/blob/55f9b9b8706f780a6fe60e2d0144cdf2248fc089/tools/support/release/pr_pin_watch.rs), and
> [protected workflow selection](https://github.com/ubugeeei-prod/vize/blob/55f9b9b8706f780a6fe60e2d0144cdf2248fc089/tools/support/release/pr_pin_gates.rs).

The following records the operator's introduction on 2026-10-09. Its historical
setup and gate descriptions are superseded by the current behavior above.

Third-party reports #8328 and #8329 require successful publication after their
actual protected merges. The local environment cannot resolve the genuine Moon
registry or perform GitHub operations from the official command's children.
The manual `release-operator.yml` entry moves that same public command to a
networked Actions runner; it does not replace the release protocol.

Only this repository's `main` can use the `release-operator` environment.
Configure its deployment branch policy to the exact `main` branch before adding
`RELEASE_OPERATOR_TOKEN`. The credential belongs to the existing maintainer
user, is provided only to the official operator step, and must match the actual
dispatch actor and the current triggering actor on reruns. Both the entry and the unchanged official driver verify the
maintain/admin role; source PR author checks remain unchanged. The workflow's
ordinary token has read-only contents permission. No registry publishing token,
App token, actor override or generated credential supplies maintainer authority.

The runner uses the existing pinned Vite+, Node, Rust 1.99.0, rust-script and Moon setup,
installs the exact frozen JS dependency graph and runs a genuine `moon update`.
It invokes `vp run release minor -y --pin` for a fresh cut, or
`vp run release --resume SOURCE_PR --pin` for the same receipt. Its separate
concurrency group cannot block the downstream Release run that it watches.

For a hosted start, `VIZE_RELEASE_EXPECTED_CUT_SHA` is the genuine workflow SHA.
The official starter validates that exact lowercase SHA immediately after
fetching current main and refuses an advanced main before creating an isolated
source worktree or preparing metadata. Redispatch from fresh main instead of
using a dependency graph installed from older bytes. Once C is frozen, ordinary
main advances remain allowed. Every pinned source, resume and metadata
integration worktree installs its own requested frozen graph; arbitrary root
`node_modules` carries no durable installation receipt and is never borrowed.
A hosted resume installs H's graph, and metadata preparation installs its own
current-main graph. The existing legacy worktree helper retains its behavior.

The optional `VIZE_RELEASE_OPERATOR_TIMEOUT_SECONDS` accepts only a positive
decimal integer up to the existing 28,800-second watch limit. The hosted entry
uses 14,400 seconds, starting before pinned preparation, with two hours of
runner setup and cleanup margin. Without the variable, the existing eight-hour
watch remains. Deadline checks precede lease acquisition, source/integration
preparation and ref writes, dispatch/retry and tag promotion; polling waits use
only their remaining budget. Expiration returns an ordinary error through the
existing `Operator::Drop`, whose exact force-with-lease cleanup removes only its
own operator ref. The source, immutable pin, metadata PR and identified Release
run remain available to the same-PR resume path.

This is a cooperative deadline. Existing blocking preparation, Git/GitHub
operations and lease cleanup retain their failure/ambiguity semantics; an
external kill is not a safe substitute. Ambiguous cleanup preserves the owner
receipt and remote-head/ref for explicit ownership recovery. Neither the
separate 20,400-second downstream promotion gate nor the source/build/preflight
budgets inherit the operator-only variable. No expired watch authorizes a tag
or claims publication. Real local Git regressions cover normal timeout cleanup,
unchanged source/pin/main, safe reacquisition and preservation of a changed
foreign lease. Shell-entry controls cover authority, input rejection and the
exact public start/resume invocations.

The operator regression is registered in the audited release-contract selector.
Its complete literal import closure and existing release input set cover the
workflow, standalone Rust driver, helper modules and synthetic Git fixtures.
Selector controls retain the five unresolved release tests' broad inputs and
prove that operator authority and source changes select the new contract.

All immutable C/H/R custody, exact-head checks, five full source gates, complete
builds and preflight, protected version metadata admission, annotated tag
ownership and public artifact acceptance remain required. The metadata PR is
admitted by its maintainer after qualification; the entry never auto-merges it.
Workflow delivery alone does not complete either third-party issue: successful
public versions and installed-artifact reproductions still have to be recorded.
