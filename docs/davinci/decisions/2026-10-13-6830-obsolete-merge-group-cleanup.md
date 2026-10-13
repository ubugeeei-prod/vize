# Cancel superseded merge-group workflow runs

Related issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

At 2026-10-13 14:53 UTC, the repository had 126 active or queued workflow
runs. Of 68 merge-group runs, 37 belonged to heads absent from the eight
current queue candidates: 20 Check, 16 Nuxt 3, and one Musea run. The queue
contained 22 PRs and allowed eight concurrent candidate builds.

Check and the consumer workflows use the synthetic queue branch as their
concurrency key. Rebuilding a candidate creates a different branch, so the
new run does not cancel the obsolete one. A sampled obsolete Check run
[38415921214](https://github.com/ubugeeei-prod/vize/actions/runs/38415921214)
still had six running 32-vCPU jobs and 16 queued tooling shards while live
candidate [38416383391](https://github.com/ubugeeei-prod/vize/actions/runs/38416383391)
had eight jobs waiting. These observations identify wasted capacity; they
are not a measured before/after speedup.

Use a short workflow triggered by Check requests and completions to cancel
only active `merge_group` runs on the main queue whose temporary branch is
deleted or no longer points at their head. Immediately before cancellation,
refresh the complete current queue and retain every head still present.
Current refs, PR checks, scheduled runs and release dispatches are retained.
Incomplete queue evidence or API errors stop cancellation.

The cleanup workflow executes only code from the trusted default branch,
uses a separate hosted runner, and serializes its own executions. It reads
workflow metadata without downloading candidate artifacts or executing PR
code. Each completed decision is retained in an artifact even if a later
API request fails. No test, required status, queue capacity or release gate
changes. Cancellation on GitHub is asynchronous, so API acceptance does not
prove that a runner has already stopped.

GitHub currently supports only `checks_requested` for the
[`merge_group` Actions trigger](https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#merge_group).
The cleanup therefore listens to the existing Check workflow, rather than
depending on a nonexistent Actions trigger for destroyed merge groups.
