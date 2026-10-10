# Release registry transport diagnosis

Diagnostic follow-up to [#8436](https://github.com/ubugeeei-prod/vize/issues/8436)
and its merged repair [#8439](https://github.com/ubugeeei-prod/vize/pull/8439).
The new failure was observed while qualifying
[#8472](https://github.com/ubugeeei-prod/vize/pull/8472).

## Exact original failure and ancestry

[Full Check 38046457258](https://github.com/ubugeeei-prod/vize/actions/runs/38046457258)
ran source `0e0618822494ef9883489b29002c47d38e16dc5c`. The unchanged controlling-terminal
release-abort law failed at `tests/tooling/task-shell.test.ts:167`: exit 124
instead of 1, after a 25.098-second PTY deadline and before the release prompt.
The full scripts result was 6,925 passed, one failed and 64 skipped.

That source contains the actual #8439 merge
`c6570cb8706dd9a4f16be984c7ecd04bdb57b26d`. On inspection, remote main was
`b2ed3149cedc4b937ce651089bb5c9b513a02bd4`, which also contains that merge.
The original task-shell test, PTY helper, registry wrapper, task commands,
release task and MoonBit setup action are byte-identical between those heads.
The failure is not explained by omission of the merged repair.

The same full scripts job passed the real 10,000-file Moon registry pipe
regression in 5.330 seconds. Its later PTY deadline captured the actual
`moon:update` child and a Git descendant chain with `futex_wait_queue`,
`do_wait`, `pipe_read` and `do_select` waits. No `pipe_write` was observed in
that deadline snapshot. The original transport child was labeled `other`.
This observation does not identify the underlying cause or network endpoint.

The complete original log is retained at
`/tmp/vize-native-0e-full-test-scripts-8472.log`, SHA256
`4b22fde55184d23f4018f644df3a37aa8de22b2a248844121639e6b135d28844`.
A local diagnostic copy of the cached registry, using the current wrapper,
actual pinned Moon binary and existing public index endpoint, completed in
2.637 seconds. Its complete streams, toolchain and index identities are
retained under `/tmp/vize-task-shell-registry-lifecycle-20261010/`.
That local success neither repairs nor explains the original Linux failure.

## Decision

Add passive Linux observations before selecting a producer repair. Keep the
original command, input, controlling terminal, prompt-triggered answer,
exit/text/Git-state law and 25/30-second deadlines exact. Add fixed Git
operation and transport labels plus anonymous pipe/socket inode identities
to the existing bounded process/session/foreground-group records. Record
neither raw arguments, input, environment, regular-file paths nor socket
peers. Keep the existing record/process bounds, examine at most 256 fd entries
and emit at most 64 channel identities per process. Explicitly close the
directory iterator at either bound. These observations do not change producer
behavior.

Identify the actual Git subcommand only after consuming supported
[Git global options](https://git-scm.com/docs/git). A config value such as
`pull`, or a `-C` directory named `fetch`, must not become a transport label.
Unknown option or command shapes retain the fixed `git` label. Moon labels
likewise describe the actual first operation rather than later argument values.

The diagnostic regression checks fixed labels against private argument
values. Neighbor laws retain actual subcommand labels when option and config
values resemble commands; a real directory fixture checks both scan bounds
and explicit closure. A real Linux pipe/socket fixture checks shared pipe
ownership and excludes a regular file carrying a private path. The original
helper fails the diagnostic phase law; the initially reviewed helper fails
both new neighbor laws. The corrected helper passes all three local laws.
The Linux ownership fixture still needs Actions execution.

## Remaining work

- Pair this decision with one concise diagnostic note on the existing #8436
  issue. This follow-up neither reopens that delivered fix nor asserts a new
  producer cause.
- Capture the unchanged original PTY law on Linux with this passive evidence.
- Identify the actual producer cause from retained transport and channel
  ownership; add a deterministic regression and repair only a demonstrated
  defect. A blanket rerun or later green result is insufficient.
- Obtain exact-head Actions and protected delivery before claiming a repair
  or unblocking the affected full qualification. The original failed run
  remains failed; no cause, waiver or release acceptance is asserted.
