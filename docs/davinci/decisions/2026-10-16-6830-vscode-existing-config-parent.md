# Prepare the existing VS Code project's directory before watching

Issue: [#6830](https://github.com/ubugeeei-prod/vize/issues/6830).

Full [Check 38887139884](https://github.com/ubugeeei-prod/vize/actions/runs/38887139884/job/116927521772)
on `99caba071ada01a251c3618683fcfe44701ecfa0` failed the unchanged
`packages/app/tsconfig.app.json` content-edit control. The mock server received
one initialization with `{ typecheck: false }`; changing the existing file to
`{"compilerOptions":{"strict":true}}` produced no restart within 20 seconds.
The retained Vize output has no watcher callback for any `packages/app` create,
change, or delete, including the earlier discovery cases. Independent
`findFiles` discovery still found the nested file. Root configuration callbacks
and their restarts succeeded. The real-server host was consequently skipped.

This is the same boundary previously reported in
[38112485518](https://github.com/ubugeeei-prod/vize/actions/runs/38112485518) and
[38131169292](https://github.com/ubugeeei-prod/vize/actions/runs/38131169292), as
recorded in the [observer diagnosis](2026-10-11-8371-editor-config-observer-diagnosis.md)
and [event trace decision](2026-10-11-8398-vscode-config-event-trace.md).
The original watcher also passed
[38728223947](https://github.com/ubugeeei-prod/vize/actions/runs/38728223947/job/116399247471).
Its retained output contains all seven nested create/change/delete callbacks.
A passing retry alone does not repair this intermittent boundary.

## Pinned watcher behavior

VS Code 1.107.1's
[string-glob watcher](https://github.com/microsoft/vscode/blob/1.107.1/src/vs/workbench/api/common/extHostFileSystemEventService.ts#L126-L130)
subscribes to the existing workspace watch without installing its own watch.
That release [pins Parcel watcher](https://github.com/microsoft/vscode/blob/1.107.1/package.json#L77)
at `1ca032aa8339260a8a3bcf825c3a1a71e3e43542`.
Its [Linux inotify implementation](https://github.com/parcel-bundler/watcher/blob/1ca032aa8339260a8a3bcf825c3a1a71e3e43542/src/linux/InotifyBackend.cc#L164-L182)
attaches a watch to a newly created directory but does not scan already-created
descendants. Recursive creation of `packages/app` can create `app` before the
`packages` event is serviced; that leaves subsequent `app` events unwatched.
[Upstream issue 243](https://github.com/parcel-bundler/watcher/issues/243)
reports the corresponding Linux new-directory race in VS Code.

This pinned source mechanism explains the observed complete absence of nested
events. The failed artifact does not contain native inotify ordering, so it is
not a captured syscall-level reproduction. No timer, generation, configuration
discovery, or server startup refusal appears after the content edit.

## Bounded fixture repair and validation

Prepare only the empty `packages/app` parent before launching VS Code, so its
initial recursive snapshot sees the existing project directory. Every original
configuration file is still created by its test case after activation. The
seven discovery cases, three content edits, file deletions, excluded dependency
control, explicit-disable controls, complete initialization packets, exactly-one
restart checks, 20-second deadline, and 100-ms polling remain unchanged. The
production extension and watcher are unchanged.

The existing actual-runner execution test checks that `packages/app` is empty
when VS Code would launch and that no configuration marker was pre-created.
It covers success, host failure, and retention failure. Removing the directory
preparation in an isolated runner mutation fails with `ENOENT` for that parent.
All 11 runner, original watcher, and synchronization execution controls pass on
the repository's Node 24.14.0. Scoped formatting, lint, and diff checks pass.
This local control verifies fixture ordering, not Linux inotify behavior.
Fresh full Actions execution must still pass the unmodified mock-host and
real-server suites.

TODO (#6830): qualify configuration observation in directories recursively
created after editor startup once the upstream watcher boundary is repaired or
a separately justified production solution exists. This fixture change does
not claim that behavior is fixed, nor does it add polling, extra watchers,
retries, or timeout allowances to the extension.
