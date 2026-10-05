# Patterned runtime failure evidence

Issue: [#7951](https://github.com/ubugeeei-prod/vize/issues/7951), paired with
[#6239](https://github.com/ubugeeei-prod/vize/issues/6239).

## First mandatory failure

Full Check [37256849378](https://github.com/ubugeeei-prod/vize/actions/runs/37256849378)
at release candidate `256cb6c0e35804bf714ab8c3d1e299b58bb08a0c`, attempt 1,
failed its source-coverage job
[111595669351](https://github.com/ubugeeei-prod/vize/actions/runs/37256849378/job/111595669351).
The unfiltered workspace coverage test failed
`match_preserves_hosts_inside_authored_loop_scopes`, VDOM backend, in
`crates/vize_atelier_vapor/tests/patterned_template_runtime.rs:104`.
That binary reported 23 passed and 1 failed; Cargo exited 101.

The first raw log is retained with SHA-256
`c9fe7938c49677fa79170f0ad0e4e438bbb4b83febc2c2fbf2a089367e0a08e1`.
The existing Rust assertion preserves generated code and stderr, but omits the
child exit code, signal, stdout and original case. Stderr is empty. The actual
Node/runtime cause remains **unknown**; neither a rendered-output mismatch nor
an unsettled cleanup promise is established. A local dependency reuse probe
failed before executing the scenario and provides no hosted runtime credit.

The owned supported watcher was stopped and Release 37256674760 was cancelled.
There was no tag, promotion or publication. The original first gate outcomes
remain historical. Fresh Miri, Docs, Fuzz and SemVer successes at this candidate
cannot transfer to a future source head. The release Matrix retains its existing
record-only/skip policy; strict ecosystem divergences remain a separate issue.
The source queue reopened for independently reviewed, exact-head-green P0
corrections while this necessary runtime investigation proceeds.

## Diagnostic decision

Keep every original patterned template, case, backend, expected tree, update,
read count and compiler option. Preserve the complete child status, code, signal,
stdout and stderr together with authored source, cases and generated code for
nonzero exits, malformed JSON and mismatched successful protocol results.

The runner records its backend, fixture index and last awaited phase. On an
unfinished process exit it writes that evidence synchronously to stderr. Mounted
runtime build/import, reactive mount/update/unmount ticks and DOM cleanup have
separate phase names. A controlled rejection followed by an unsettled cleanup confirms another
diagnostic gap: the original error can remain captured inside the cleanup
wrapper without reaching stderr before exit. Preserve that primary error in
failure-only evidence before cleanup; a reporter failure cannot replace it.
This controlled reproduction does not establish the historical coverage cause.
Completed successful runs retain exactly the existing
`{passed:N}` stdout and emit no new exit evidence. No compiler, runtime package,
coverage threshold, corpus or instruction ceiling changes are part of this
change.

Deterministic genuine child-process laws cover silent exit 13, Unix signal
termination, malformed/mismatched successful stdout, an unsettled top-level
await, a rejection followed by pending cleanup, a failing reporter and exact
completed success output. A controlled unresolved promise's
exit 13 validates the diagnostic; it does not establish the historical failure's
cause. Controlled failure streams are written synchronously before forced exit, so
these laws do not depend on pending stdout flushes. Existing cleanup and
observer laws remain intact.

## Required follow-through

Replay the original runtime scenario on Actions at the new diagnostic head,
including the full source-coverage lane. Preserve the first outcome. If it fails,
use the actual status/streams/phase to make the smallest necessary correction,
with the original case and output expectations retained. A successful diagnostic
run alone is not proof that the earlier cause has been fixed.

Only qualified necessary corrections and the selected ready P0 batch may enter
the recovered release. After their actual protected merges, resume PR #7811
through the supported command, refresh from literal main and rerun all official
exact-candidate gates. Verify registries, native/editor assets, tag/main and real
Pages deployment before claiming public completion. No blind candidate retry,
waiver, manual version/tag/candidate mutation or stale-SHA gate transfer.
