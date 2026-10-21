# Legacy deletion readiness (#6854)

The deletion decision covers five products: compiler, linter, formatter,
typechecker and LSP. This gate is an explicit `workflow_dispatch` audit while
native product adapters and historical evidence are incomplete. Ordinary PRs,
pushes and scheduled `main` runs do not execute a permanently red readiness
job. A deletion cannot claim readiness from a green ordinary Check run. The
ordinary PR and merge-group `check-js` job guards removal or renaming of a
legacy crate's `Cargo.toml`: a successful readiness audit must exist at both
the exact base SHA and deletion candidate SHA. It also guards the retained DOM,
SSR and Vapor compatibility emitter route witnesses. Removing or relocating a
witness from its production route requires the same exact-SHA audits, while
ordinary selector improvements that preserve the retained route do not. Missing
audits or a change to either SHA fails closed. Other PRs pay only the source
custody check.

The gate checks the exact deletion candidate with T1 merge-group fixture
results and an on-demand T2 Real Project Matrix run at the same SHA. It also
checks successful scheduled T2 observations over an approved stability period.
The Real Project Matrix currently runs weekly; `minimumStableDays` and
`maximumGapDays` are explicit policy fields rather than assuming daily runs.
The policy and pinned 150-project registry SHA must remain fixed throughout
the stability period. The workflow verifies the evidence bundle's producer is
a successful exact-SHA Real Project Matrix run, and the evaluator checks the
recorded T1 and T2 run IDs against the Actions API.

For each product, tier and observation, the authored target set must be covered
by exact `(case ID, target)` result rows bound to the raw manifest digest and
source SHA. All cases must be active and have verified per-case dialect labels;
all fourteen canonical dialect labels must appear for each product. T2 must
include every pinned project for each product. A project-level presence claim
or input-only rare-dialect source is not a product result. Every row needs a
completed legacy observation, native handling certified by the shared
whole-product classifier, a verified source-build receipt and runtime
observation, and an adapter-verified product comparison. Missing, unsupported,
fallback, unknown and legacy-backed work fails the gate.

The current policy pins the target sets exposed by the published acceptance
adapters: compiler DOM/SSR/Vapor, linter `lint`, formatter `fmt`, typechecker
`batch` and LSP `stdio`. A checked-in contract rejects drift between those
targets and their manifests. This fixes the authored target denominator; it
does not claim that the registered fixtures are complete product coverage.
The stability duration/cadence remain unset. The formatter now has the first
production observation/comparison verifier. It binds the source-built CLI
receipt to every Real Project Matrix shard, requires all 150 registered
projects' formatter checks, and requires the non-empty projects' check/write
agreement, idempotence, parse preservation and lint agreement artifacts.
Differences, waivers, unusable baselines, missing files and partial shard
identity remain blockers. The other four products still have no verifier, and
no `davinci-deletion-evidence` producer exists. The manual gate therefore exits
nonzero and names these blockers; it grants no deletion credit.
These are remaining #6854 tasks, alongside product adapter and case coverage
work in #6853, #6891 and #6892. Before any deletion is permitted, wire the
trusted Real Project Matrix bundle producer to actual per-run result artifacts,
review the policy values and require a successful exact-candidate readiness
run for the deletion change. The guard now closes the recorded compiler
route-swap gap by keeping custody of the DOM, SSR and Vapor compatibility
emitters and of both production `vize build` compiler entry points. A route
swap in either the ordinary build path or its statistics path therefore needs
successful readiness audits at the exact base and candidate SHAs. Native-only
compiler, lint, typechecker and LSP adapters and complete formatter
candidate/history evidence remain unfinished, so #6854 stays open. This gate
does not certify legacy deletion.
