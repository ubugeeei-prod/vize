# VS Code configuration event trace

This bounded diagnostic change belongs to the existing configuration Stack,
including [#8398](https://github.com/ubugeeei-prod/vize/pull/8398). Its source
foundation is `3f0f3940ce6f68bf56cadd540006c15a94e2a328`.

The genuine original editor host gate on `bd77ac6637a7f59d80a9a9c7ecdd6076ff783608`
failed in Full Check `38131169292`, job `114445643299`. Editing the existing
`packages/app/tsconfig.app.json` contents did not restart the mocked server
within the original 20 seconds with native typecheck watchers disabled. The
complete server packet proves no restart during that interval, but does not
prove whether the extension received the file event or completed configuration
discovery. The lower foundation and failed upper source have identical complete
VS Code extension and editor Action trees; that establishes source custody,
not a passing lower-head qualification.

The existing Vize OutputChannel receives narrow trace lines only under the
already existing host-test command environment switch. The actual production
watcher and configuration synchronization paths report callback kind, URI and
acceptance, timer scheduling/cancellation/firing, synchronization generation,
findFiles entry/completion/error, generation refusal, and client start/stop
boundaries. Logging refusal leaves the original lifecycle result intact.
Configuration values and process environments are not added to this trace.

The string watcher glob, excluded directories, search limit, 150ms timer,
generation comparisons, initialization options, client publication, original
test contents and assertions, 20-second deadline, and disabled native watchers
remain unchanged. No additional watcher, request, polling, retry, refresh,
pipeline stage, timeout or restart is introduced. The complete existing archive
reader tail moves verbatim in a separate move-only commit so the extension
entry file shrinks while new modules remain within 350 lines.

[Pinned VS Code 1.107.1 OutputChannel source](https://github.com/microsoft/vscode/blob/1.107.1/src/vs/workbench/api/common/extHostOutput.ts)
already writes ordinary output channels into native `output_logging_*/N-Vize.log`
files. The host runner copies only complete Vize log files after the host exits
and before its existing profile cleanup. Multiple sessions retain distinct
relative paths. Missing files, symbolic links or copy failures produce an
explicit retention refusal; diagnostic failure does not replace the original
host error, and profile cleanup still runs. The existing artifact step retains
this additive directory on success and failure, with both original failure
directories and every original workload command preserved.

Local source laws compare the complete extension AST after removing precisely
the trace additions and reversing the literal archive move. They also compare
the complete search assignment, callback predicate/subscriptions, runner,
Action and original suite/fixture trees. Actual source execution controls cover
accepted and excluded events, pending/rejected discovery, displaced generations,
timer cancellation, deactivate during stop, logger refusal, complete log bytes,
cleanup on success/failure/copy refusal, and moved archive output/error behavior.
The affected local suite passes 256 tests with no skips; scoped type-aware lint,
extension no-emit type checking and the original source growth gate pass.

These are local/source observations. This instrumentation has not identified or
fixed the intermittent original 20-second failure. The sole configuration owner
must compose the source once and retain the next original host trace through
the unchanged Actions gate before selecting an essential cause repair. A
retention refusal or absent boundary remains missing evidence, never a waiver.
