# Run the Rust aggregate with native Node

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The fresh push at `08f798af09931b4f6339a0a5422f13feaf80dfbc` passed all nine
ordinary workflows. [Check 38901193268](https://github.com/ubugeeei-prod/vize/actions/runs/38901193268)
took 295 seconds from creation to its required report; the complete workflow
span was 301 seconds, including managed Code Quality's earlier start. All
18,187 selected Rust tests and Clippy passed. Its tested merge was
`5997d7c3702a20f96ccbffa887000c9b86abbcf8`, with actual base
`a39af7c010ce94f93da6173a3877d1c5e063cf86`.

After the final Rust cohort completed, three strict aggregate jobs occupied
another 37 seconds. The Rust report took 16 seconds, including two seconds for
Vite+ bootstrap caching and seven for setup-vp; its actual Rust-tier assertion
took less than one reported second. The other aggregate jobs already used
setup-node, taking two seconds to provision Node.

Use the same pinned Node setup for the Rust report and disable package-manager
cache discovery. Its required-tier validator, source-bound worker selector and
typechecker aggregate import only Node builtins and repository modules. The
queue's TypeScript aggregate therefore runs directly through Node's native
type stripping, without Vite+ or an install. Keep the same job dependencies,
assertions, failure/cancellation rules, exact-source worker selection, complete
fixture observations and artifact publication.

Existing strict-gate, worker-selection, typechecker transport and workflow
contracts run under Node 24.14.0. Hosted normal-PR and merge-queue execution
remain the acceptance authority. The seven-second setup difference is an
observed opportunity, not a promised saving or a five-minute guarantee.

## Fresh hosted result

[Check 38902375238](https://github.com/ubugeeei-prod/vize/actions/runs/38902375238)
at `e4ad440e0b53224952d97cb731621cc0ba3ee513` passed on its first attempt.
The tested merge was `8ae32f8cb265a9549be0e0bfca25f9294abe6963`, with base
`163c349e00cc49d117da0a24b2d5acf26aa3c9b1`. Check ran from 12:42:07 UTC
to the required report's completion at 12:46:34: **267 seconds**. All nine
ordinary workflows, including managed Code Quality's earlier 12:42:01 start,
completed in **273 seconds**. No jobs were reused or rerun.

All 18,188 Rust test identities passed with no duplicates. Every identity from
the previous run remains; main's `v-pre` regression test accounts for the one
addition. Clippy, doctests and all strict aggregates passed. The Rust report
took eight seconds instead of 16, and the final product-to-report interval was
18 seconds instead of 37. The critical product changed from Compiler to JS
builders, so the full 28-second improvement cannot be attributed to this edit.

This run used 76.3 runner-minutes across 48 jobs, versus 61.5 in the preceding
fresh run. Sources, workloads and cache state differ. The five-minute target
was observed for this run; it is not a latency guarantee or a demonstrated
runner-cost reduction. Exact-source full and protected merge-queue validation
remain separate acceptance requirements.
