# Executable publication for Corsa process fixtures

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

The original [source-coverage failure](https://github.com/ubugeeei-prod/vize/actions/runs/38064580844/job/114249529043)
at `03cf6ccc9fe2d130056d75a6f0ced0fce2aaa209` remains a failed run:
1773 tests passed, one failed, two were ignored, and Cargo exited 101.
`failed_and_timed_out_original_overlays_are_reaped_without_source_writes`
failed during its initial API spawn with `Text file busy (os error 26)`.
It did not reach the later overlay timeout, source-preservation or reap assertions.
The log does not identify the retained descriptor holder or the loop iteration.

Each fixture already has its own temporary directory. Closing its local writer
before spawning does not establish that every inherited writer has closed.
[Rust #114554](https://github.com/rust-lang/rust/issues/114554) describes the
fork/exec interval in which another child retains the same writable open-file
description. This mechanism fits the source; the historical holder remains unknown.

The test-only publication helper now locks the writer exclusively, closes it,
and acquires a shared lock through an independently reopened read-only file.
An inherited descriptor retains the same lock until its last writer closes;
unrelated child processes need no lock-aware code. Do not explicitly unlock the
writer before closing it. Read-only descriptors inherited after the barrier
cannot keep an executable write-open. The helper uses the existing standard
library file-lock API within the workspace's Rust 1.95 minimum version.

Linux controls retain a real writable duplicate in an owned child at fd 2 and
verify its device/inode against the fixture. The close-only control requires one
exec to fail with exact `ETXTBSY`, then releases and reaps that child and requires
one successful exec. The guarded control observes actual lock contention and
pending publication, releases the same child, and requires successful publication
and one successful exec with the unchanged bytes and executable mode. The child
emits a literal readiness acknowledgement and accepts a separate release input.

All existing fixture scripts, backend arguments, assertions, timeout values,
configuration mutations, source preservation and PID/reap checks remain intact.
Only the three wrapper-publication sites in the original-program failure tests
use the helper. No Corsa API, product path, dependency, retry, global process lock
or workflow timeout changes. Other fixture families are outside this change.

Validation must run the new controls and unchanged Corsa laws on Actions,
including source coverage at the corrected source head, before protected queue
admission. A successful successor qualifies that source; it does not reclassify
the original failure or prove its historical cause.
