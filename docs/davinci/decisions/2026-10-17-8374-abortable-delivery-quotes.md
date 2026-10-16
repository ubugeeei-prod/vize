# Abortable delivery quotes

Issue: #8374. One missing public-entry demo, not completion of the whole audit.

The existing `useAsyncState` profile example demonstrates loading, errors and
latest-result state. `useAsyncResource` additionally owns native abort signals,
retained data, explicit cancellation/reset and non-rejecting execution outcomes;
it had no live reader example.

Add one complete public-import SFC for selecting a delivery quote. All data is
explicitly local sample data. Real timers delay producers; the native abort
signal clears each timer and settling removes its listener. Response and abort
counters make cancellation distinct from merely ignoring a late result. The
last execution label exposes success, error, cancelled and superseded results.

Reuse the current source-owned preview and existing SSR/hydration/source identity
checks. Native keyboard interactions at desktop/mobile widths verify physical
timer cancellation through response counts after the deadline, replacement,
retained data during pending/failure, recovery, reset and focus. After initial
and interacted screenshots, unmount the actual hydrated Vue app and advance past
its pending producer; late browser errors or unhandled rejection fail the build.
The existing `async-resource.test.ts` scope-stop case separately verifies that
scope disposal aborts the native signal and resolves the active execution as
cancelled. No mock browser host or test-only SFC hooks are added.

The composable owns cancellation on scope disposal; the producer owns its timer
and abort-listener cleanup. The example explains replacing that producer with
`fetch(url, { signal })` without claiming a real server integration. No runtime
API, default preset, navigation or other-language translation is changed.
