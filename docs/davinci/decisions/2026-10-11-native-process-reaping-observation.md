# Native process retirement observation

Protected Check [38146270848](https://github.com/ubugeeei-prod/vize/actions/runs/38146270848)
failed the original physical native drop law in the #8650 candidate
`047cab281c8421f14ffdb9f5bae17e8aca5f8b7c`. Its owned Linux `/proc/<pid>/stat`
read returned `ESRCH` after the existence probe; 4556 of 4557 Rust tests passed.
The observer recognized `ENOENT` but panicked on the equally terminal missing
process result. The candidate was removed from the queue. Linux's
[proc stat reader](https://github.com/torvalds/linux/blob/v6.18/fs/proc/base.c#L746-L770)
returns `ESRCH` when the recorded process task has disappeared.

Read the recorded process stat directly. Only missing-path `ENOENT` and Linux
missing-process `ESRCH` establish disappearance. Every other read failure
remains fatal, including permissions, I/O failure and interruption. Removing
the existence probe also prevents its implicit suppression of read errors.

Every observable stat byte reaches the original birth-identity check. An
unchanged recorded birth remains live until retirement; a zombie still gets
no reaping credit. PID reuse proves retirement only when the recorded birth
differs. The five-second deadline, existing sleep interval, actual native
executable/owner/configuration checks and all original native fixture assertions
remain unchanged. This changes only the test observer, with no product or SDK
behavior change and no retries or skipped checks.

Three additive laws pin the error boundary and unchanged live/zombie stat
contents. The missing-process law fails under the original policy and all
three pass after the fix on Rust 1.99.0. Exact-head Actions must execute the
original Linux physical reuse, configuration invalidation, shutdown and drop
laws. Full protected queue and actual merge remain required; the local decoder
laws alone do not establish physical native reaping or performance improvement.
