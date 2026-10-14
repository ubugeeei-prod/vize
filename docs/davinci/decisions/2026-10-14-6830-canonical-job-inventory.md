# Read canonical worker metadata through the workflow check suite

The merge candidates for [#8887](https://github.com/ubugeeei-prod/vize/pull/8887),
[#8903](https://github.com/ubugeeei-prod/vize/pull/8903) and
[#8922](https://github.com/ubugeeei-prod/vize/pull/8922) reached the full canonical
observers, but the finalizers in runs 38462344413 and 38462345375 failed while
listing Actions jobs with HTTP 502. The existing bounded GET retries did not
recover. The same runs' check-suite inventories and individual Actions jobs
remained available. This is a provider metadata failure, not a corpus mismatch.

For [#6830](https://github.com/ubugeeei-prod/vize/issues/6830), enumerate the exact
workflow run's check suite with all pages and `filter=all`, then fetch only the
canonical observers through the individual Actions job API. Resolve job IDs
from the same repository/run's details URLs; check IDs need not equal job IDs.
Bind each job response to its check URL, run, head and observer name. Request
`checks: read` only through the reusable canonical finalizer's caller chain.

Checks provide inventory, never successful-execution evidence. Retain all
attempts and the unchanged validation of steps, latest execution, inherited
execution signatures, runner identity, artifact digests and upload windows.
Missing steps, newer pending or failed attempts, unknown workers and foreign
metadata still fail. The run and suite identities are checked again after
selection. No corpus, instruction budget or required check is removed.

Regression controls exercise paginated histories with different check/job IDs,
retained pending and unknown observers, and mismatched URL/run/head metadata.
Existing canonical selection and artifact tests continue to enforce the
execution contract. Exact-head Actions and the protected merge queue validate
the real provider path before merge.

The new reader was also exercised against failed run 38462345375: all three
observers returned their 27 steps, and the unchanged selector accepted the
three matching artifacts. This validates metadata recovery; downloaded
artifact validation and a successful queue run remain separate merge gates.
