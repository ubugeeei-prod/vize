# Legacy deletion readiness (#6854)

The deletion decision covers five products: compiler, linter, formatter,
typechecker and LSP. This gate is an explicit `workflow_dispatch` audit while
native product adapters and historical evidence are incomplete. Ordinary PRs,
pushes and scheduled `main` runs do not execute a permanently red readiness
job. A deletion cannot claim readiness from a green ordinary Check run.

The gate checks the exact deletion candidate with T1 merge-group fixture
results and an on-demand T2 Real Project Matrix run at the same SHA. It also
checks successful scheduled T2 observations over an approved stability period.
The Real Project Matrix currently runs weekly; `minimumStableDays` and
`maximumGapDays` are explicit policy fields rather than assuming daily runs.
The policy and pinned 146-project registry SHA must remain fixed throughout
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

The current policy deliberately leaves the target sets for four products and
the stability duration/cadence unset. No production native observation or
comparison verifier is registered, and no `davinci-deletion-evidence` producer
exists. The formatter's current native result is unsupported. The manual gate
therefore exits nonzero and names these blockers; it grants no deletion credit.
These are remaining #6854 tasks, alongside product adapter and case coverage
work in #6853, #6891 and #6892. Before any deletion is permitted, wire the
trusted Real Project Matrix bundle producer to actual per-run result artifacts,
review the policy values and require a successful exact-candidate readiness
run for the deletion change. This first gate does not certify legacy deletion.
