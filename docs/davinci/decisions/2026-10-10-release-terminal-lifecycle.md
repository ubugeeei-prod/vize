# Release confirmation terminal lifecycle

Issue: [#8436](https://github.com/ubugeeei-prod/vize/issues/8436).

The unchanged controlling-terminal release-abort law failed on two independent
source heads: #8381, Check 38029035846 / job 114145965341, and #8399, Check
38029026150 / job 114145927362. Both exited 124 after approximately 25 seconds;
the expected status is 1. Their output stops after the Vite+ command containing
`moon update && moon run ... release -- minor`, before the confirmation prompt.
Both jobs use Node 24.14.0, Ubuntu 24.04 and the pinned MoonBit compiler.

## Decision

Keep the actual controlling terminal, prompt-triggered answer, exit status,
`Aborted.` text, unchanged head/status/tags, 25-second PTY deadline and
30-second parent timeout. Capture bounded Linux process/session/foreground-group
metadata without arguments, input or environment values. Find the actual waiting
producer before choosing a repair. A green subsequent attempt does not explain
the failed attempts or complete this issue.

The diagnostic branch temporarily added a manual-only job to the registered
native capture workflow and retained metadata from original source tooling.
Both workflow files are restored byte for byte in the final change, including
their 350-line budget, original hooks, commands and prerequisites. The helper
now prints bounded metadata only on a real deadline; investigations can also
request retained lifecycle files. The active 0.440 publication authority and
public manifests remain untouched.

## Reproduced producer defect and repair

The installed Moon CLI revision `4da23f805e562bcdb20a45764f1ab12cb892bf1d`
[waits for Git pull with both output streams piped](https://github.com/moonbitlang/moon/blob/4da23f805e562bcdb20a45764f1ab12cb892bf1d/crates/mooncake/src/update.rs)
without draining them. A local-origin stale registry fixture creates the same
10,000-file fast-forward for both executions. A normally drained Git pull emits
730,081 stdout bytes and 1,187 stderr bytes and finishes in 1.88 seconds. The
real Moon update blocks with its Git processes still present and no forwarded
output, even after the registry HEAD has advanced. A loopback proxy refuses the
independent best-effort symbols download, so external network availability is
excluded from this reproduction.

Run only the release task's existing Moon registry-update invocation with an
additional process-local `merge.stat=false` Git setting. This suppresses Git's
per-file diffstat and created-file summary; the same drained Git update emits
39 stdout bytes and 1,208 stderr bytes. The real Moon update then terminates and
retains the exact origin HEAD and Git tree. The actual update and index remain
mandatory before release execution. No repository or global Git config changes.

Preserve inherited Git configuration entries and append to their validated count.
If inherited command-line config parameters exist, retain their exact bytes and
append the same constant presentation setting because those parameters otherwise
override the counted entries. Apply this environment only to the update child;
the subsequent release process inherits the original environment. Preserve exit
status, signal propagation and inherited terminal descriptors.

The real fixture saves complete stdout, stderr, toolchain identity, waiting
processes, seed/target HEAD and Git tree under
`target/vize-tests/moon-registry-update/`. Its red case terminates only its owned
process group. The green case must retain the byte-identical workload and exact
Git identities; it does not substitute a fake Moon command or change the
original PTY deadlines.

This is a scoped workaround for the reproduced successful-pull output deadlock.
TODO: remove the presentation wrapper after a pinned Moon CLI drains both output
streams and the same real stale-index regression proves the repair.

## Evidence and remaining work

- Fresh-main macOS, Node 24.14.0 and pinned MoonBit: the original law passed in
  10.32 seconds with a cold native module, and 2.97 seconds after instrumentation.
  Neither result attributes the Linux failure to Node, MoonBit or the terminal.
- Diagnostic source 7a5105d258f8c8ab7af7401e01ff74b770d5f40f, Actions
  38030432750 / job 114150031838: the four planned original-law samples passed.
  Their retained process evidence shows foreground groups throughout.
- Full source tooling, diagnostic source
  50cf3b8f6269983fd514b107cfb385ba8742cda5, Check 38030637690 / job
  114150723434: the original law passed in 1.077 seconds. The diagnostic workflow
  additions failed existing 350-line budget assertions, which remain unchanged;
  restoring the complete original workflows addresses that diagnostic failure.
- The historical failed jobs restored a 125,919,237-byte MoonBit cache archive;
  this passing branch restored 125,972,351 bytes under the same key. A stale
  cached index is a plausible connection to the reproduced defect, but the
  historical waiting process was not recorded. Do not claim that attribution.
- TODO: verify the real stale-index regression and unchanged original PTY law
  on Linux Actions, pass the complete exact-head source checks and protected
  merge queue, and verify the actual merge. #8436 remains open until then.
