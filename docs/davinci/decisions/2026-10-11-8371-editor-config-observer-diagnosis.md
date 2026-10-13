# Existing editor configuration event diagnosis

[Issue #8371](https://github.com/ubugeeei-prod/vize/issues/8371) remains
unqualified for the original configuration content-change host control.
The exact composed source `fc4c7a8e43f2480b856e19baf4bf02c1c0f58918`
failed [full Check 38112485518](https://github.com/ubugeeei-prod/vize/actions/runs/38112485518)
at `existing-config-smoke.cjs:124`: an existing nested
`packages/app/tsconfig.app.json` edit did not restart the mocked LSP with
`typecheck: false`. The original 20-second deadline and 100-ms polling
remain unchanged. This is a mocked-host refusal; the real host and later
editor consumers were skipped.

The original failure upload only covered the real host, which had not
started. No complete mocked event artifact exists; the raw failed job
retains only the last ten events. That evidence does not prove whether
the native event was lost, the glob filtered it, or the lifecycle failed.
The first traced source `dc0de59837` passed the complete editor composite
in [38113203604](https://github.com/ubugeeei-prod/vize/actions/runs/38113203604).
Its passive watcher probes can affect timing; this does not prove a cause
or qualify a production fix. The next control restores the production
watcher byte for byte and retains each content-edit control's complete
before/after LSP messages and authored configuration, including successful
controls. Failure custody also keeps the mocked workspace and raw events.
No expectation, original 20-second deadline, or 100-ms poll changes.

The diagnostic starts from exact config bottom
`04b15e087766bfcf29662b01e829fa79c1931100`, whose editor source is preserved
in the failed composition. Its temporary hosted workflow executes only
the unchanged original editor-host job and complete editor composite; it
is not a complete Check qualification and must not enter the merge queue.
The temporary probes and scoped workflow will be excluded from any source
fix handed to the config owner. A production cause and the original
positive and refusal controls must be qualified before readiness.

The faithful watcher control `aaef834399` passed the complete editor
composite in [38113920929](https://github.com/ubugeeei-prod/vize/actions/runs/38113920929).
Artifact `11692487779` retains all three edits' complete before/after
messages and authored configs; each original control has exactly one
replacement initialization with `{ typecheck: false }` and unchanged
shutdown/exit ownership. The original failure cause is still unproved.

An independent startup ownership candidate remains unqualified: the
original `startClient` publishes `client` only after asynchronous
initialization. An overlapping explicit disable can observe no client,
finish as disabled, and then receive a stale startup completion. A new
controlled original initialize gate checks pending disable and ordinary
completion with the full initialization options, original 20-second
poll helper, exact shutdown/exit messages, and the owned child process
exit. Its before/after messages are retained. This does not yet establish
the cause of the original nested-file content-edit failure.

The first pending-startup attempt
[38114194553](https://github.com/ubugeeei-prod/vize/actions/runs/38114194553)
failed on VS Code download transport before the host. The next
[38114363742](https://github.com/ubugeeei-prod/vize/actions/runs/38114363742)
reached the new test but exposed its incorrect expectation of recommended
defaults after an explicit typecheck switch. Correcting only that new
complete packet expectation to the existing `{ typecheck: false }`
contract retains every shutdown/exit assertion and the original controls.

The corrected original source `b4b713fcdd` reproduced the actual startup
ownership bug in [38114588493](https://github.com/ubugeeei-prod/vize/actions/runs/38114588493):
after explicit disable, the gated LSP child `34811` initialized and served
requests but sent no shutdown/exit and did not terminate in the unchanged
20-second control. Artifact `11693196691` retains its complete before/after
messages. All original configuration content-edit controls ran first.

The bounded candidate gives each configuration synchronization an owned
generation. Only the current generation may publish selected executable
evidence, a client, or ready/failure state after asynchronous operations.
A superseded startup stops its own initialized client instead of replacing
the current client. Deactivation also invalidates pending generations and
the original debounce timer. Initialization and shutdown timeouts are
unchanged; a server that never completes initialization retains the existing
startup liveness limitation. This is not claimed to explain the historical
nested configuration event failure. A move-only extraction preserves all
original server/status types and fixture response fields while keeping the
existing large files smaller; no source-length allowance is increased.
The composed candidate must pass both pending-disable and ordinary-startup
controls plus the complete original editor consumers before source readiness.

The exact candidate `cdb64aabb1` passed the complete original editor
composite in [38114928136](https://github.com/ubugeeei-prod/vize/actions/runs/38114928136).
Artifact `11692748429` retains both controls: each keeps the whole
`{ typecheck: false }` initialization packet, exactly one shutdown and exit,
and the actual owned child process termination (pending-disable `35073`,
ordinary completion `35173`). The control bytes match the original-source
before test. All original content-edit controls also pass. The source
handoff excludes the diagnostic-only Check workflow; the configuration
Stack owner must qualify the actual composed fresh-main source and full
protected gates before merge/release. No issue completion is claimed.
